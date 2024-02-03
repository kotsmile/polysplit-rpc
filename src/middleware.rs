use lazy_static::lazy_static;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Status;
use rocket::outcome::Outcome::*;
use rocket::request::{self, FromRequest};
use rocket::{Data, Request};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

lazy_static! {
    static ref REQUEST_COUNTS: Mutex<HashMap<String, (u64, u64)>> = Mutex::new(HashMap::new());
}

pub struct RateLimiter;

#[rocket::async_trait]
impl Fairing for RateLimiter {
    fn info(&self) -> Info {
        Info {
            name: "Rate Limiter",
            kind: Kind::Request,
        }
    }

    async fn on_request(&self, request: &mut Request<'_>, _: &mut Data<'_>) {
        let ip = request
            .client_ip()
            .map(|ip| ip.to_string())
            .unwrap_or_default();
        let mut counts = REQUEST_COUNTS.lock().unwrap();
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let count_info = counts.entry(ip).or_insert((0, current_time));
        if current_time - count_info.1 > 60 {
            *count_info = (1, current_time);
        } else {
            count_info.0 += 1;
            if count_info.0 > 25 {
                request.local_cache(|| RateLimitExceeded(true));
            }
        }
    }
}

#[derive(Default)]
pub struct RateLimitExceeded(bool);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for RateLimitExceeded {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        match request.local_cache(|| RateLimitExceeded(false)).0 {
            true => Error((Status::TooManyRequests, ())),
            false => Success(RateLimitExceeded(false)),
        }
    }
}
