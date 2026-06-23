use actix_governor::governor::clock::{Clock, DefaultClock, QuantaInstant};
use actix_governor::governor::NotUntil;
use actix_governor::{KeyExtractor, SimpleKeyExtractionError};
use actix_web::dev::ServiceRequest;
use actix_web::{HttpResponse, HttpResponseBuilder};
use std::fmt::{Display, Error, Formatter};
use std::net::IpAddr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RateLimitKey {
    Ip(IpAddr),
}

impl Display for RateLimitKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            RateLimitKey::Ip(ip) => write!(f, "ip:{}", ip),
        }
    }
}

#[derive(Debug, Clone)]
pub struct IpKeyExtractor;

impl KeyExtractor for IpKeyExtractor {
    type Key = RateLimitKey;
    type KeyExtractionError = SimpleKeyExtractionError<&'static str>;

    fn extract(&self, req: &ServiceRequest) -> Result<Self::Key, Self::KeyExtractionError> {
        if let Some(real_ip) = req.headers().get("X-Real-IP") {
            if let Ok(ip_str) = real_ip.to_str() {
                if let Ok(ip) = ip_str.parse::<IpAddr>() {
                    return Ok(RateLimitKey::Ip(ip));
                }
            }
        }

        if let Some(forwarded) = req.headers().get("X-Forwarded-For") {
            if let Ok(forwarded_str) = forwarded.to_str() {
                if let Some(first_ip) = forwarded_str.split(',').next() {
                    if let Ok(ip) = first_ip.trim().parse::<IpAddr>() {
                        return Ok(RateLimitKey::Ip(ip));
                    }
                }
            }
        }

        req.peer_addr()
            .map(|socket| RateLimitKey::Ip(socket.ip()))
            .ok_or_else(|| SimpleKeyExtractionError::new("Could not extract IP"))
    }

    fn exceed_rate_limit_response(
        &self,
        negative: &NotUntil<QuantaInstant>,
        mut response: HttpResponseBuilder,
    ) -> HttpResponse {
        let wait_time = negative
            .wait_time_from(DefaultClock::default().now())
            .as_secs();

        response
            .json(serde_json::json!({
                "error": true,
                "message": format!("You have made too many requests. Please try again in {} seconds.", wait_time),
                "retry_after": wait_time
            }))
    }

    fn whitelisted_keys(&self) -> Vec<Self::Key> {
        vec![]
    }
}