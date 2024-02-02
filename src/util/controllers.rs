use log::error;
use rocket::{
    http::{ContentType, Status},
    response::{self, Responder, Response},
    serde::json::Json,
    Request,
};
use rocket_okapi::{
    gen::OpenApiGenerator, okapi::openapi3::Responses, response::OpenApiResponderInner,
    OpenApiError,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
pub struct ResponseError {
    pub error: String,
    #[serde(skip)]
    pub status: Status,
}

impl OpenApiResponderInner for ResponseError {
    fn responses(_gen: &mut OpenApiGenerator) -> Result<Responses, OpenApiError> {
        let mut responses = Json::<ResponseError>::responses(_gen)?;

        let mut response_value = None;
        if let Some((_response_code, response)) = responses.responses.iter().next() {
            response_value = Some(response.clone());
        }
        responses.responses.clear();
        if let Some(response_value) = response_value {
            responses.responses.insert("400".to_owned(), response_value);
        }

        Ok(responses)
    }
}

impl std::fmt::Display for ResponseError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.error,)
    }
}

impl std::error::Error for ResponseError {}

impl<'r> Responder<'r, 'static> for ResponseError {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        let body = serde_json::to_string(&self).unwrap();
        Response::build()
            .sized_body(body.len(), std::io::Cursor::new(body))
            .header(ContentType::JSON)
            .status(self.status)
            .ok()
    }
}

impl From<rocket::serde::json::Error<'_>> for ResponseError {
    fn from(err: rocket::serde::json::Error<'_>) -> Self {
        use rocket::serde::json::Error::*;
        match err {
            Io(io_err) => {
                error!("IO error: {io_err}");
                ResponseError {
                    error: "IO Error".to_owned(),
                    status: Status::UnprocessableEntity,
                }
            }
            Parse(_raw_data, _parse_error) => ResponseError {
                error: "Parse Error".to_owned(),
                status: Status::UnprocessableEntity,
            },
        }
    }
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ResponseData<T> {
    data: T,
}

impl<T> ResponseData<T> {
    pub fn build(data: T) -> Json<Self> {
        Json(Self { data })
    }
}

pub type ResponseResult<T> = Result<Json<T>, ResponseError>;
pub type ResponseResultData<T> = ResponseResult<ResponseData<T>>;
pub type RequestResult<'a, T> = Result<Json<T>, rocket::serde::json::Error<'a>>;
