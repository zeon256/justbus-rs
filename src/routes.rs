//! All functions are kept seperate for ease of reading
//! and to make it easier to add or remove stuff

use crate::errors::JustBusError;
use actix_web::{web, HttpResponse};
use lta::r#async::bus::get_arrival;
use lta::r#async::lta_client::LTAClient;

#[cfg(feature = "cht")]
use cht_time::Cache as ChtCache;

#[cfg(feature = "hashbrown")]
use hashbrown_time::Cache as HashBrownCache;

#[cfg(feature = "dashmap")]
use dashmap_time::Cache as DashCache;

#[cfg(feature = "hashbrown")]
use parking_lot::RwLock;

use crate::TTL;
use actix_web::body::Body;
use justbus_utils::InternalEntry;
use serde::export::fmt::Debug;
use std::hash::Hash;
use std::time::Instant;

type JustBusResult = Result<HttpResponse, JustBusError>;

pub async fn dummy() -> &'static str {
    "hello_world"
}

#[cfg(feature = "cht")]
pub async fn get_timings(
    bus_stop: web::Path<u32>,
    lru: web::Data<ChtCache<u32, String>>,
    client: web::Data<LTAClient>,
) -> JustBusResult {
    let bus_stop = bus_stop.into_inner();
    let in_lru = lru.get(&bus_stop);

    let res = match in_lru {
        Some(f) => HttpResponse::Ok().content_type("application/json").body(f),
        None => {
            let arrivals = get_arrival(&client, bus_stop, None)
                .await
                .map_err(JustBusError::ClientError)?
                .services;

            let arrival_str = serde_json::to_string(&arrivals).unwrap();
            let _ = lru.insert(bus_stop, arrival_str.clone());

            HttpResponse::Ok()
                .content_type("application/json")
                .body(arrival_str)
        }
    };

    Ok(res)
}

#[cfg(feature = "hashbrown")]
pub async fn get_timings(
    bus_stop: web::Path<u32>,
    lru: web::Data<RwLock<HashBrownCache<u32, String>>>,
    client: web::Data<LTAClient>,
) -> JustBusResult {
    let bus_stop = bus_stop.into_inner();
    let lru_r = lru.read();
    let in_lru = lru_r.get(bus_stop);

    let res = match in_lru {
        Some(f) => HttpResponse::Ok().content_type("application/json").body(f),
        None => {
            // drop the lock
            drop(lru_r);

            let arrivals = get_arrival(&client, bus_stop, None)
                .await
                .map_err(JustBusError::ClientError)?
                .services;

            let mut lru_w = lru.write();
            let arrival_str = serde_json::to_string(&arrivals).unwrap();

            let _ = lru_w.insert(bus_stop, arrival_str.clone());

            HttpResponse::Ok()
                .content_type("application/json")
                .body(arrival_str)
        }
    };

    Ok(res)
}

#[cfg(feature = "dashmap")]
pub async fn get_timings(
    bus_stop: web::Path<u32>,
    lru: web::Data<DashCache<u32, String>>,
    client: web::Data<LTAClient>,
) -> JustBusResult {
    let bus_stop = bus_stop.into_inner();

    let in_lru = lru.get(&bus_stop);

    let res = match in_lru {
        Some(f) => HttpResponse::Ok()
            .content_type("application/json")
            .body(&f.value),
        None => {
            let arrivals = get_arrival(&client, bus_stop, None)
                .await
                .map_err(JustBusError::ClientError)?
                .services;

            let arrival_str = serde_json::to_string(&arrivals).unwrap();
            let _ = lru.insert(bus_stop, arrival_str.clone());

            HttpResponse::Ok()
                .content_type("application/json")
                .body(arrival_str)
        }
    };

    Ok(res)
}
