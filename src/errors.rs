use actix_web::{HttpResponse, ResponseError};
use std::fmt::Formatter;

#[derive(Debug)]
pub enum JustBusError {
    ClientError(lta::utils::LTAError),
    CacheError,
}

impl std::fmt::Display for JustBusError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Internal Server Error")
    }
}

impl ResponseError for JustBusError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::InternalServerError().finish()
    }
}
