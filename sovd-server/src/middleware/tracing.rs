// SPDX-FileCopyrightText: Copyright Liebherr-Digital Development Center GmbH
// SPDX-License-Identifier: Apache-2.0
//

use std::future::{Ready, ready};
use std::pin::Pin;

use actix_web::Error;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready};

const TARGET: &str = "srv::http";

#[derive(Debug, Clone, Default)]
pub(crate) struct Tracing;

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

pub(crate) struct TracingService<S> {
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
        let fut = self.service.call(req);

        Box::pin(async move {
            let start = std::time::Instant::now();
            let result = fut.await;
            let elapsed = start.elapsed();

            match &result {
                Ok(response) => {
                    let status = response.status().as_u16();
                    tracing::info!(
                        target: TARGET,
                        method = %method,
                        path = %path,
                        status_code = status,
                        elapsed = format!("{:?}", elapsed)
                    );
                }
                Err(err) => {
                    let status = err.as_response_error().status_code().as_u16();
                    tracing::error!(
                        target: TARGET,
                        method = %method,
                        path = %path,
                        status_code = status,
                        elapsed = format!("{:?}", elapsed),
                        error = %err
                    );
                }
            }
            result
        })
    }
}
