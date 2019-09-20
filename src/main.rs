use actix_web::{
    web, App, Error as ActixErr, HttpRequest, HttpResponse, HttpServer, Responder, ResponseError,
};
use futures::future::{ok as fut_ok, Either};
use lru_time_cache::LruCache;
use lta::bus::bus_arrival::ArrivalBusService;
use lta::r#async::{bus::get_arrival, lta_client::LTAClient, prelude::*};
use serde::Serialize;
use std::fmt::Formatter;
use parking_lot::RwLock;
use std::{env::var, time::Duration};

#[derive(Clone)]
struct LruState {
    pub lru: LruCache<u32, TimingResult>,
    pub client: LTAClient,
}

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

impl LruState {
    pub fn new(lru: LruCache<u32, TimingResult>, client: LTAClient) -> Self {
        LruState { lru, client }
    }
}

fn get_timings(
    lru_state: web::Data<RwLock<LruState>>,
) -> impl Future<Item = HttpResponse, Error = JustBusError> {
    let lru_state_2 = lru_state.clone();
    let inner = lru_state.read();
    let in_lru = inner.lru.peek(&83139);

    match in_lru {
        Some(data) => {
            println!("Taking from LRU");
            Either::A(fut_ok(
                HttpResponse::Ok().json(TimingResult::new(83139, data.clone().data)),
            ))
        }
        None => {
            println!("Fresh data from LTA");
            Either::B(
                get_arrival(&inner.client, 83139, None)
                    .then(move |r| {
                        r.map(|r| {
                           let data = r.services.clone();
                            let mut lru_w = lru_state_2.write();
                            lru_w.lru.insert(83139, TimingResult::new(83139, data));
                            r
                        })
                    })
                    .map(|f| HttpResponse::Ok().json(TimingResult::new(83139, f.services)))
                    .map_err(|e| JustBusError::ClientError(e)),
            )
        }
    }
}

type LruCacheU32 = LruCache<u32, TimingResult>;

fn main() {
    println!("Starting server @ 127.0.0.1:8080");
    HttpServer::new(move || {
        let api_key = var("API_KEY").unwrap();
        let ttl = Duration::from_millis(1000 * 15);
        let lru_state = RwLock::new(LruState::new(
            LruCacheU32::with_expiry_duration(ttl),
            LTAClient::with_api_key(api_key),
        ));
        App::new()
            .route("/api/v1/timings", web::get().to_async(get_timings))
            .data(lru_state)
    })
    .bind("127.0.0.1:8080")
    .unwrap()
    .run()
    .unwrap();
}
