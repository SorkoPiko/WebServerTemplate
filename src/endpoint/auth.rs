use std::time::Duration;
use actix_web::{post, web, FromRequest, HttpRequest, HttpResponse};
use actix_web::dev::Payload;
use anyhow::Context;
use chrono::Utc;
use futures_util::future::{err, ok, Ready};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use crate::AppState;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenClaims {
    pub id: i64,
    pub exp: i64,
    pub iat: i64,
}

// TODO: replace with actual auth
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct AuthQuery {
    pub id: i64,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct AuthResponse {
    pub token: String,
}

#[utoipa::path(summary = "Create token", responses(
    (status = OK, description = "Create token", body = AuthResponse)
))]
#[post("/create")]
pub async fn create(
    app_state: web::Data<AppState>,
    query: web::Json<AuthQuery>,
) -> Result<HttpResponse, actix_web::Error> {
    let token = create_token(query.id, &app_state.keys.jwt_secret, Duration::from_hours(24))
        .context("Failed to create token")
        .map_err(|e| {
            log::error!("{e:?}");
            actix_web::error::ErrorInternalServerError("Token generation error")
        })?;

    Ok(HttpResponse::Ok().json(AuthResponse { token }))
}

pub fn create_token(id: i64, secret: &str, duration: Duration) -> anyhow::Result<String> {
    let now = Utc::now();
    let claims = TokenClaims {
        id,
        exp: (now + duration).timestamp(),
        iat: now.timestamp(),
    };

    let encoding_key = EncodingKey::from_secret(secret.as_bytes());
    let token = encode(&Header::default(), &claims, &encoding_key).context("Failed to encode token")?;

    Ok(token)
}

pub fn verify_token(token: &str, secret: &str) -> anyhow::Result<TokenClaims> {
    let decoding_key = DecodingKey::from_secret(secret.as_bytes());
    let validation = Validation::new(Algorithm::HS256);

    let data = decode::<TokenClaims>(token, &decoding_key, &validation)
        .context("Failed to decode token")?;
    let current_time = Utc::now().timestamp();
    if data.claims.exp < current_time {
        anyhow::bail!("Token expired");
    }

    Ok(data.claims)
}

impl FromRequest for TokenClaims {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        if let Some(auth_header) = req.headers().get("Authorization") {
            if let Ok(auth_str) = auth_header.to_str() {
                if auth_str.starts_with("Bearer ") {
                    let token = auth_str[7..].to_string();
                    let app_state = match req.app_data::<web::Data<AppState>>() {
                        Some(state) => state.clone(),
                        None => return err(actix_web::error::ErrorInternalServerError("App config missing")),
                    };
                    return match verify_token(&token, &app_state.keys.jwt_secret) {
                        Ok(claims) => ok(claims),
                        Err(_) => err(actix_web::error::ErrorUnauthorized("Invalid token"))
                    }
                }
            }
        }
        err(actix_web::error::ErrorUnauthorized("Missing token"))
    }
}