use axum::{
    extract::{State, Request, ConnectInfo},
    http::{StatusCode},
    middleware::Next,
    response::Response,
};
use moka::sync::Cache;
use std::{net::IpAddr, time::{Duration, Instant}};
use crate::state::AppState;

const MAX_REQUESTS_PER_MINUTE: u32 = 5;
const WINDOW_DURATION: Duration = Duration::from_secs(60);

#[derive(Clone)]
pub struct RateLimiter {
    cache: Cache<IpAddr, (u32, Instant)>,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            cache: Cache::builder()
                .time_to_live(WINDOW_DURATION)
                .build(),
        }
    }

    pub fn is_limited(&self, ip: IpAddr) -> bool {
        let mut count = 0;
        let mut last_request_time = Instant::now();

        if let Some((c, t)) = self.cache.get(&ip) {
            count = c;
            last_request_time = t;
        }

        if last_request_time.elapsed() > WINDOW_DURATION {
            // Reset count if window has passed
            self.cache.insert(ip, (1, Instant::now()));
            false
        } else if count < MAX_REQUESTS_PER_MINUTE {
            // Increment count if within limit
            self.cache.insert(ip, (count + 1, last_request_time));
            false
        } else {
            // Exceeded limit
            true
        }
    }
}

pub async fn rate_limit_middleware(
    State(app_state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let ip = request.extensions()
        .get::<ConnectInfo<std::net::SocketAddr>>()
        .map(|ConnectInfo(addr)| addr.ip())
        .or_else(|| {
            request.headers()
                .get("X-Forwarded-For")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.split(',').next())
                .and_then(|s| s.trim().parse::<IpAddr>().ok())
        })
        .ok_or(StatusCode::BAD_REQUEST)?;

    if app_state.rate_limiter.is_limited(ip) {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    Ok(next.run(request).await)
}
