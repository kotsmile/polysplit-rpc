use schemars::JsonSchema;
use serde::Serialize;

#[derive(Debug, Clone, Copy, JsonSchema, Serialize)]
pub struct Monitoring {
    pub income_requests: u128,
    pub success_income_requests: u128,
    pub error_income_requests: u128,
}

impl Monitoring {
    pub fn new() -> Self {
        Self {
            income_requests: 0,
            success_income_requests: 0,
            error_income_requests: 0,
        }
    }
}
