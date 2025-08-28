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

//! HTTP request tracing middleware for structured logging.

use std::future::{Ready, ready};
use std::pin::Pin;

use actix_web::Error;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready};
use tracing::Instrument;

#[derive(Debug, Clone, Default)]
pub struct Tracing;

impl Tracing {
    pub fn new() -> Self {
        Self
    }
}

impl<S, B> Transform<S, ServiceRequest> for Tracing
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = TracingService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(TracingService { service }))
    }
}

pub struct TracingService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for TracingService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let method = req.method().to_string();
        let path = req.path().to_string();

        let span = tracing::info_span!(
            "req",
            method = %method,
            path = %path,
            status_code = tracing::field::Empty,
            latency = tracing::field::Empty,
        );

        let _enter = span.enter();
        drop(_enter);

        let fut = self.service.call(req);
        let span_clone = span.clone();

        Box::pin(
            async move {
                let start = std::time::Instant::now();
                let result = fut.await;
                let latency = start.elapsed();

                match &result {
                    Ok(response) => {
                        let status = response.status();
                        span_clone.record("status_code", status.as_u16());
                        span_clone.record("latency", format!("{:?}", latency));
                    }
                    Err(err) => {
                        let status_code = err.as_response_error().status_code().as_u16();
                        span_clone.record("status_code", status_code);
                        span_clone.record("latency", format!("{:?}", latency));
                        tracing::error!(
                            latency = ?latency,
                            error = %err,
                        );
                    }
                }
                result
            }
            .instrument(span),
        )
    }
}
