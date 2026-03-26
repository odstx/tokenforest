use async_trait::async_trait;
use axum::{
    extract::{FromRequestParts, Request},
    http::{request::Parts, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::{
    headers::{Authorization, authorization::Bearer},
    TypedHeader,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::models::User;

pub const JWT_SECRET: &[u8] = b"your-secret-key-change-in-production";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: i64,
    pub username: String,
    pub exp: usize,
}

#[derive(Debug, Serialize)]
pub struct AuthError {
    pub message: String,
}

pub fn create_jwt_token(user: &User) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize
        + 60 * 60 * 24;

    let claims = Claims {
        sub: user.id,
        username: user.username.clone(),
        exp: expiration,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET),
    )
}

pub fn verify_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET),
        &Validation::default(),
    );
    token_data.map(|data| data.claims)
}

pub async fn auth_middleware(
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = bearer.token();
    
    match verify_token(token) {
        Ok(claims) => {
            request.extensions_mut().insert(claims);
            Ok(next.run(request).await)
        }
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}

#[derive(Debug, Clone)]
pub struct CurrentUser(pub Claims);

#[async_trait]
impl<S> FromRequestParts<S> for CurrentUser
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<Claims>()
            .cloned()
            .map(CurrentUser)
            .ok_or(StatusCode::UNAUTHORIZED)
    }
}
