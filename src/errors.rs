use actix_web::{HttpResponse, ResponseError};
use lta::LTAError;
use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum JustBusError {
    #[error("Client error: {0}")]
    ClientError(#[from] LTAError),

    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
}

impl ResponseError for JustBusError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::InternalServerError().finish()
    }
}
