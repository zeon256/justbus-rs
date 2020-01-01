extern crate jemallocator;
use crate::hashmap::{Cache};
use actix_web::{web, App, HttpResponse, HttpServer, ResponseError, Responder};
use lta::{
    prelude::*,
    r#async::{
        lta_client::LTAClient,
        bus::get_arrival
    },
};
use std::fmt::Formatter;
use std::{env::var, time::Duration};
use actix_web::dev::Service;

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
            let arrivals =  get_arrival(&client, bus_stop, None)
                .await
                .map_err(JustBusError::ClientError)?;
            let arrival_str = serde_json::to_string(&arrivals).unwrap();
            lru.insert(bus_stop, arrival_str.clone());
            HttpResponse::Ok().content_type("application/json").body( arrival_str)
        }
    };

    Ok(res)
}

async fn dummy() -> impl Responder {
    "hello_world"
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    println!("Starting server @ 127.0.0.1:8080");
    let api_key = var("API_KEY").expect("NO API_KEY FOUND!");
    let ttl = Duration::from_millis(1000 * 15);
    let client = LTAClient::with_api_key(api_key);
    HttpServer::new(move || {
        App::new()
            .route("/api/v1/dummy", web::get().to(dummy))
            .route(
                "/api/v1/timings/{bus_stop}",
                web::get().to(get_timings),
            )
            .data(client.clone())
            .data(Cache::<u32, String>::with_ttl(ttl))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
