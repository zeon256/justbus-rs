use actix_web::{HttpResponse, ResponseError};
use lta::utils::LTAError;
use std::fmt;

#[derive(Debug)]
pub enum JustBusError {
    ClientError(lta::utils::LTAError),
}

impl From<lta::utils::LTAError> for JustBusError {
    fn from(e: LTAError) -> Self {
        JustBusError::ClientError(e)
    }
}

impl fmt::Display for JustBusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Client error! {}", self.to_string())
    }
}

impl ResponseError for JustBusError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::InternalServerError().finish()
    }
}
