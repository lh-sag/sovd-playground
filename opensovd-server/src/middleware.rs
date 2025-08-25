//
// Copyright (c) 2025 The Contributors to Eclipse OpenSOVD.
//
// See the NOTICE file(s) distributed with this work for additional
// information regarding copyright ownership.
//
// This program and the accompanying materials are made available under the
// terms of the Apache License Version 2.0 which is available at
// https://www.apache.org/licenses/LICENSE-2.0
//
// SPDX-License-Identifier: Apache-2.0
//

/// HTTP tracing middleware layer inspired by tower-http's TraceLayer.
///
/// This middleware provides structured tracing for HTTP requests with customizable
/// span creation, request/response logging, and error handling.
///

use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::Error;
use std::future::{ready, Ready};
use std::pin::Pin;

use std::time::Duration;
use tracing::{Instrument, Span};

pub trait MakeSpan: Send + Sync {
    fn make_span(&self, request: &ServiceRequest) -> Span;
}

pub trait OnRequest: Send + Sync {
    fn on_request(&self, request: &ServiceRequest, span: &Span);
}

pub trait OnResponse<B>: Send + Sync {
    fn on_response(&self, response: &ServiceResponse<B>, latency: Duration, span: &Span);
}

#[derive(Debug, Clone, Default)]
pub struct DefaultOnResponse;

impl<B> OnResponse<B> for DefaultOnResponse {
    fn on_response(&self, response: &ServiceResponse<B>, latency: Duration, span: &Span) {
        let status = response.status();
        span.record("status_code", status.as_u16());
        
        let message = if status.is_success() {
            "HTTP request completed successfully"
        } else if status.is_client_error() {
            "HTTP request completed with client error"
        } else if status.is_server_error() {
            "HTTP request completed with server error"
        } else {
            "HTTP request completed"
        };
        
        tracing::debug!(
            latency = ?latency,
            status_code = %status.as_u16(),
            "{}",
            message
        );
    }
}

impl<B, F> OnResponse<B> for F
where
    F: Fn(&ServiceResponse<B>, Duration, &Span) + Send + Sync,
    B: 'static,
{
    fn on_response(&self, response: &ServiceResponse<B>, latency: Duration, span: &Span) {
        self(response, latency, span)
    }
}

pub trait OnFailure: Send + Sync {
    fn on_failure(&self, error: &Error, latency: Duration, span: &Span);
}

#[derive(Debug, Clone)]
pub struct DefaultMakeSpan {
    level: tracing::Level,
}

impl Default for DefaultMakeSpan {
    fn default() -> Self {
        Self {
            level: tracing::Level::DEBUG,
        }
    }
}


impl MakeSpan for DefaultMakeSpan {
    fn make_span(&self, request: &ServiceRequest) -> Span {
        macro_rules! make_span {
            ($level:ident) => {
                tracing::span!(
                    tracing::Level::$level,
                    "request",
                    method = %request.method(),
                    path = %request.path(),
                    status_code = tracing::field::Empty,
                )
            };
        }

        match self.level {
            tracing::Level::TRACE => make_span!(TRACE),
            tracing::Level::DEBUG => make_span!(DEBUG),
            tracing::Level::INFO => make_span!(INFO),
            tracing::Level::WARN => make_span!(WARN),
            tracing::Level::ERROR => make_span!(ERROR),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct DefaultOnRequest;

impl OnRequest for DefaultOnRequest {
    fn on_request(&self, _request: &ServiceRequest, _span: &Span) {
        tracing::debug!("HTTP request started processing");
    }
}

#[derive(Debug, Clone, Default)]
pub struct DefaultOnFailure;

impl OnFailure for DefaultOnFailure {
    fn on_failure(&self, error: &Error, latency: Duration, span: &Span) {
        span.record("error", &tracing::field::display(error));
        tracing::error!(
            latency = ?latency,
            error = %error,
            "HTTP request failed"
        );
    }
}

#[derive(Debug, Clone)]
pub struct TraceLayer<
    MakeSpan = DefaultMakeSpan,
    OnRequest = DefaultOnRequest,
    OnResponse = DefaultOnResponse,
    OnFailure = DefaultOnFailure,
> {
    make_span: MakeSpan,
    on_request: OnRequest,
    on_response: OnResponse,
    on_failure: OnFailure,
}

impl<B> OnResponse<B> for () {
    fn on_response(&self, _response: &ServiceResponse<B>, _latency: Duration, _span: &Span) {}
}

impl Default for TraceLayer {
    fn default() -> Self {
        Self::http()
    }
}

impl TraceLayer {
    pub fn http() -> Self {
        TraceLayer {
            make_span: DefaultMakeSpan::default(),
            on_request: DefaultOnRequest,
            on_response: DefaultOnResponse,
            on_failure: DefaultOnFailure,
        }
    }
}

impl<MakeSpan, OnResponse, OnRequest, OnFailure> TraceLayer<MakeSpan, OnRequest, OnResponse, OnFailure> {
    pub fn make_span_with<NewMakeSpan>(
        self,
        make_span: NewMakeSpan,
    ) -> TraceLayer<NewMakeSpan, OnRequest, OnResponse, OnFailure> {
        TraceLayer {
            make_span,
            on_request: self.on_request,
            on_response: self.on_response,
            on_failure: self.on_failure,
        }
    }

    pub fn on_request<NewOnRequest>(self, on_request: NewOnRequest) -> TraceLayer<MakeSpan, NewOnRequest, OnResponse, OnFailure> {
        TraceLayer {
            make_span: self.make_span,
            on_request,
            on_response: self.on_response,
            on_failure: self.on_failure,
        }
    }

    pub fn on_response<NewOnResponse>(
        self,
        on_response: NewOnResponse,
    ) -> TraceLayer<MakeSpan, OnRequest, NewOnResponse, OnFailure> {
        TraceLayer {
            make_span: self.make_span,
            on_request: self.on_request,
            on_response,
            on_failure: self.on_failure,
        }
    }

    pub fn on_failure<NewOnFailure>(
        self,
        on_failure: NewOnFailure,
    ) -> TraceLayer<MakeSpan, OnRequest, OnResponse, NewOnFailure> {
        TraceLayer {
            make_span: self.make_span,
            on_request: self.on_request,
            on_response: self.on_response,
            on_failure,
        }
    }
}

impl<S, B, MakeSpan, OnRequest, OnResponse, OnFailure> Transform<S, ServiceRequest>
    for TraceLayer<MakeSpan, OnRequest, OnResponse, OnFailure>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
    MakeSpan: self::MakeSpan + Clone + 'static,
    OnRequest: self::OnRequest + Clone + 'static,
    OnResponse: self::OnResponse<B> + Clone + 'static,
    OnFailure: self::OnFailure + Clone + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = TraceService<S, MakeSpan, OnRequest, OnResponse, OnFailure>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(TraceService {
            service,
            make_span: self.make_span.clone(),
            on_request: self.on_request.clone(),
            on_response: self.on_response.clone(),
            on_failure: self.on_failure.clone(),
        }))
    }
}

pub struct TraceService<S, MakeSpan, OnRequest, OnResponse, OnFailure> {
    service: S,
    make_span: MakeSpan,
    on_request: OnRequest,
    on_response: OnResponse,
    on_failure: OnFailure,
}

impl<S, B, MakeSpan, OnRequest, OnResponse, OnFailure> Service<ServiceRequest>
    for TraceService<S, MakeSpan, OnRequest, OnResponse, OnFailure>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
    MakeSpan: self::MakeSpan + Clone + 'static,
    OnRequest: self::OnRequest + Clone + 'static,
    OnResponse: self::OnResponse<B> + Clone + 'static,
    OnFailure: self::OnFailure + Clone + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let span = self.make_span.make_span(&req);

        let _enter = span.enter();
        self.on_request.on_request(&req, &span);
        drop(_enter);

        let fut = self.service.call(req);
        let on_failure = self.on_failure.clone();
        let on_response = self.on_response.clone();

        Box::pin(
            async move {
                let start = std::time::Instant::now();
                let result = fut.await;
                let latency = start.elapsed();

                match &result {
                    Ok(response) => {
                        on_response.on_response(response, latency, &tracing::Span::current());
                    }
                    Err(err) => {
                        on_failure.on_failure(err, latency, &tracing::Span::current());
                    }
                }

                result
            }
            .instrument(span),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_on_response() {
        let _on_response = DefaultOnResponse;
        assert_eq!(std::mem::size_of::<DefaultOnResponse>(), 0);
    }
}