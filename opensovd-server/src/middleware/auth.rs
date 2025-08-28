use std::future::{Future, Ready, ready};
use std::pin::Pin;

use actix_web::Error;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready};
use actix_web::http::header::AUTHORIZATION;
use derive_more::{Display, Error};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Display, Error)]
pub(crate) enum AuthError {
    #[display("Invalid JWT format")]
    InvalidFormat,

    #[display("JWT token expired")]
    Expired,

    #[display("Invalid JWT signature")]
    InvalidSignature,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Claims {
    pub sub: Option<String>,
    pub exp: Option<usize>,
    pub iat: Option<usize>,
    pub aud: Option<String>,
    pub iss: Option<String>,
    pub scope: Option<Vec<String>>,
    pub roles: Option<Vec<String>>,
}

/// JWT validator function that validates a token and returns claims
pub(crate) fn jwt_validator(
    token: &str,
    decoding_key: &DecodingKey,
    validation: &Validation,
) -> Result<Claims, AuthError> {
    match decode::<Claims>(token, decoding_key, validation) {
        Ok(token_data) => Ok(token_data.claims),
        Err(e) => {
            use jsonwebtoken::errors::ErrorKind;
            match e.kind() {
                ErrorKind::ExpiredSignature => Err(AuthError::Expired),
                ErrorKind::InvalidSignature => Err(AuthError::InvalidSignature),
                _ => Err(AuthError::InvalidFormat),
            }
        }
    }
}

/// Bearer authentication middleware for JWT tokens
#[derive(Clone)]
pub(crate) struct BearerAuth {
    decoding_key: DecodingKey,
    validation: Validation,
}

impl BearerAuth {
    /// Create a new BearerAuth middleware from an RSA public key in PEM format
    pub fn from_rsa_pem(pem: &[u8]) -> Result<Self, AuthError> {
        let decoding_key = DecodingKey::from_rsa_pem(pem).map_err(|_| AuthError::InvalidFormat)?;

        let mut validation = Validation::new(Algorithm::RS256);
        validation.validate_exp = true;
        validation.validate_aud = false;

        Ok(Self {
            decoding_key,
            validation,
        })
    }
}

impl<S, B> Transform<S, ServiceRequest> for BearerAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = BearerAuthService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(BearerAuthService {
            service,
            decoding_key: self.decoding_key.clone(),
            validation: self.validation.clone(),
        }))
    }
}

pub(crate) struct BearerAuthService<S> {
    service: S,
    decoding_key: DecodingKey,
    validation: Validation,
}

impl<S, B> Service<ServiceRequest> for BearerAuthService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let path = req.path();

        // Only protect /v1/* endpoints
        if !path.contains("/v1/") {
            let fut = self.service.call(req);
            return Box::pin(async move { fut.await });
        }

        // Check for Authorization header
        let auth_header = match req.headers().get(AUTHORIZATION) {
            Some(header) => header,
            None => {
                return Box::pin(async move {
                    Err(crate::response::ApiError::unauthorized("Missing authorization header").into())
                });
            }
        };

        // Parse Bearer token
        let auth_str = match auth_header.to_str() {
            Ok(s) => s,
            Err(_) => {
                return Box::pin(async move {
                    Err(crate::response::ApiError::unauthorized("Invalid authorization header").into())
                });
            }
        };

        if !auth_str.starts_with("Bearer ") {
            return Box::pin(async move {
                Err(crate::response::ApiError::unauthorized("Authorization must use Bearer scheme").into())
            });
        }

        let token = &auth_str[7..];

        // Validate token using the jwt_validator function
        match jwt_validator(token, &self.decoding_key, &self.validation) {
            Ok(_claims) => {
                // Token is valid, proceed with the request
                let fut = self.service.call(req);
                Box::pin(async move { fut.await })
            }
            Err(e) => {
                tracing::debug!("Token validation failed: {}", e);
                Box::pin(async move { Err(crate::response::ApiError::unauthorized(e.to_string()).into()) })
            }
        }
    }
}
