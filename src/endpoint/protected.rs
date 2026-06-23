use crate::endpoint::auth::TokenClaims;
use actix_web::{get, HttpResponse};
use serde::Serialize;

// TODO: replace with actual endpoints
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ProtectedResponse {}

#[utoipa::path(summary = "Get protected data", responses(
    (status = OK, description = "Get protected data", body = ProtectedResponse),
    (status = UNAUTHORIZED, description = "Unauthorized access")
))]
#[get("")]
pub async fn protected(
    _: TokenClaims,
) -> Result<HttpResponse, actix_web::Error> {
    Ok(HttpResponse::Ok().json(ProtectedResponse {}))
}