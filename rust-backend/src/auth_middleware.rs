use axum::{
    async_trait,
    extract::{FromRequestParts, Request},
    http::{request::Parts, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json, RequestPartsExt,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use serde_json::json;

use crate::jwt::{self, Claims, JwtConfig};

// Extractor for Claims that can be used in handler parameters
#[async_trait]
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract Authorization header
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({ "error": "Missing authorization header" })),
                )
            })?;

        // Verify JWT token
        let jwt_config = JwtConfig::default();
        let claims = jwt::verify_token(bearer.token(), &jwt_config).map_err(|e| {
            tracing::warn!("JWT verification failed: {}", e);
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({ "error": "Invalid or expired token" })),
            )
        })?;

        Ok(claims)
    }
}

// Middleware function for authentication
pub async fn auth_middleware(
    TypedHeader(auth_header): TypedHeader<Authorization<Bearer>>,
    mut req: Request,
    next: Next,
) -> Result<Response, impl IntoResponse> {
    let jwt_config = JwtConfig::default();

    match jwt::verify_token(auth_header.token(), &jwt_config) {
        Ok(claims) => {
            // Add claims to request extensions for downstream handlers
            req.extensions_mut().insert(claims);
            Ok(next.run(req).await)
        }
        Err(e) => {
            tracing::warn!("Authentication failed: {}", e);
            Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({ "error": "Invalid or expired token" })),
            ))
        }
    }
}

