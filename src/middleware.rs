use lazy_static::lazy_static;
// Global storage for request counts
#[macro_use]
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Status;
use rocket::{Data, Request, Response, Rocket};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

lazy_static! {
    static ref REQUEST_COUNTS: Mutex<HashMap<String, (u64, u64)>> = Mutex::new(HashMap::new());
}

pub struct RequestLimiter;

#[rocket::async_trait]
impl Fairing for RequestLimiter {
    fn info(&self) -> Info {
        Info {
            name: "Request Limiter",
            kind: Kind::Request | Kind::Response,
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
            // Reset every minute
            *count_info = (0, current_time);
        }
        count_info.0 += 1;
    }

    async fn on_response<'r>(&self, request: &'r Request<'_>, response: &mut Response<'r>) {
        let ip = request
            .client_ip()
            .map(|ip| ip.to_string())
            .unwrap_or_default();
        let counts = REQUEST_COUNTS.lock().unwrap();
        let count_info = counts.get(&ip);

        if let Some((count, _)) = count_info {
            if *count > 100 {
                // Limit to 100 requests per minute
                response.set_status(Status::TooManyRequests);
            }
        }
    }
}
