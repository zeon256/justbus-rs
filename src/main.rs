extern crate jemallocator;
use actix_web::{
    web, App, Error as ActixErr, HttpRequest, HttpResponse, HttpServer, Responder, ResponseError,
};
use futures::future::{ok as fut_ok, Either};
use lru_time_cache::LruCache;
use lta::bus::bus_arrival::ArrivalBusService;
use lta::r#async::{bus::get_arrival, lta_client::LTAClient, prelude::*};
use parking_lot::RwLock;
use serde::Serialize;
use std::fmt::Formatter;
use std::{env::var, time::Duration};

#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[derive(Debug)]
enum JustBusError {
    ClientError(lta::Error),
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
#[derive(Serialize, Clone)]
#[serde(rename_all(serialize = "PascalCase"))]
struct TimingResult {
    pub bus_stop_code: u32,
    pub data: Vec<ArrivalBusService>,
}

impl TimingResult {
    pub fn new(bus_stop_code: u32, data: Vec<ArrivalBusService>) -> Self {
        TimingResult {
            bus_stop_code,
            data,
        }
    }
}

impl Responder for TimingResult {
    type Error = ActixErr;
    type Future = Result<HttpResponse, ActixErr>;

    fn respond_to(self, _: &HttpRequest) -> Self::Future {
        let body = serde_json::to_string(&self.data)?;

        Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(body))
    }
}

fn get_timings(
    path: web::Path<u32>,
    lru: web::Data<RwLock<LruCacheU32>>,
    client: web::Data<LTAClient>,
) -> impl Future<Item = HttpResponse, Error = JustBusError> {
    let inner_path = path.into_inner();
    let lru_state_2 = lru.clone();
    let inner = lru.read();
    let in_lru = inner.peek(&inner_path);

    match in_lru {
        Some(data) => Either::A(fut_ok(
            HttpResponse::Ok().json(TimingResult::new(inner_path, data.clone().data)),
        )),
        None => {
            println!(
                "Fresh data from LTA. client_ptr: {:p}, cache_ptr: {:p}",
                &client, &lru
            );
            Either::B(
                get_arrival(&client, inner_path, None)
                    .then(move |r| {
                        r.map(|r| {
                            let bus_stop = inner_path;
                            let data = r.services.clone();
                            let mut lru_w = lru_state_2.write();
                            lru_w.insert(bus_stop, TimingResult::new(bus_stop, data));
                            r
                        })
                    })
                    .map(|f| {
                        HttpResponse::Ok().json(TimingResult::new(f.bus_stop_code, f.services))
                    })
                    .map_err(JustBusError::ClientError),
            )
        }
    }
}

type LruCacheU32 = LruCache<u32, TimingResult>;

fn main() {
    println!("Starting server @ 127.0.0.1:8080");
    let api_key = var("API_KEY").unwrap();
    let ttl = Duration::from_millis(1000 * 15);
    let client = web::Data::new(LTAClient::with_api_key(api_key));
    let lru_cache = web::Data::new(RwLock::new(LruCacheU32::with_expiry_duration(ttl)));

    HttpServer::new(move || {
        App::new()
            .route(
                "/api/v1/timings/{bus_stop}",
                web::get().to_async(get_timings),
            )
            .register_data(client.clone())
            .register_data(lru_cache.clone())
    })
    .bind("127.0.0.1:8080")
    .unwrap()
    .run()
    .unwrap();
}
