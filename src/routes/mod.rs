//! All functions are kept seperate for ease of reading
//! and to make it easier to add or remove stuff

pub mod bus_arrivals;

use crate::errors::JustBusError;
use actix_web::HttpResponse;

type JustBusResult = Result<HttpResponse, JustBusError>;

pub async fn health() -> &'static str {
    "hello_world"
}
