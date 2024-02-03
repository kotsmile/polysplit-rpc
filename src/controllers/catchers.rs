use rocket::catch;

#[catch(429)]
pub fn rate_limit_exceeded() -> &'static str {
    "Too many requests. Visit https://polysplit.cloud for more information."
}
