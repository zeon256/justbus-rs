extern crate jemallocator;
use crate::hashmap::Cache;
use actix_web::{web, App, HttpResponse, HttpServer, ResponseError};
use lta::{
    prelude::*,
    r#async::{bus::get_arrival, lta_client::LTAClient},
};
use std::{fmt::Formatter, time::Duration};
mod hashmap;

#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[derive(Debug)]
enum JustBusError {
    ClientError(lta::utils::LTAError),
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

async fn get_timings(
    bus_stop: web::Path<u32>,
    lru: web::Data<Cache<u32, String>>,
    client: web::Data<LTAClient>,
) -> Result<HttpResponse, JustBusError> {
    let bus_stop = bus_stop.into_inner();
    let in_lru = lru.get(bus_stop);
    let res = match in_lru {
        Some(f) => HttpResponse::Ok().content_type("application/json").body(f),
        None => {
            let arrivals = get_arrival(&client, bus_stop, None)
                .await
                .map_err(JustBusError::ClientError)?
                .services;

            let arrival_str = serde_json::to_string(&arrivals).unwrap();

            lru.insert(bus_stop, arrival_str.clone());
            HttpResponse::Ok()
                .content_type("application/json")
                .body(arrival_str)
        }
    };

    Ok(res)
}

async fn dummy() -> &'static str {
    "hello_world"
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    println!("Starting server @ 127.0.0.1:8080");
    let api_key = std::env::var("API_KEY").expect("API_KEY NOT FOUND!");
    let ttl = Duration::from_secs(15);
    let client = LTAClient::with_api_key(api_key);
    HttpServer::new(move || {
        App::new()
            .route("/api/v1/dummy", web::get().to(dummy))
            .route("/api/v1/timings/{bus_stop}", web::get().to(get_timings))
            .data(client.clone())
            .data(Cache::<u32, String>::with_ttl_and_size(ttl, 500))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
