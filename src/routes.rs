//! All functions are kept seperate for ease of reading
//! and to make it easier to add or remove stuff

use crate::errors::JustBusError;
use actix_web::{web, HttpResponse};
use lta::r#async::bus::get_arrival;
use lta::r#async::lta_client::LTAClient;

#[cfg(feature = "cht")]
use cht_time::Cache as ChtCache;

#[cfg(feature = "swisstable")]
use hashbrown_time::Cache as SwissCache;

#[cfg(feature = "swisstable")]
use parking_lot::RwLock;

#[cfg(feature = "dashmap")]
use dashmap_time::Cache as DashCache;

#[cfg(feature = "logging")]
use log::info;

type JustBusResult = Result<HttpResponse, JustBusError>;

pub async fn health() -> &'static str {
    "hello_world"
}

#[cfg(any(feature = "cht", feature = "dashmap"))]
pub async fn get_timings(
    bus_stop: web::Path<u32>,
    #[cfg(feature = "cht")] lru: web::Data<ChtCache<u32, String>>,
    #[cfg(feature = "dashmap")] lru: web::Data<DashCache<u32, String>>,
    client: web::Data<LTAClient>,
) -> JustBusResult {
    let bus_stop = bus_stop.into_inner();
    let in_lru = lru.get(&bus_stop);

    let res = match in_lru {
        #[rustfmt::skip]
        Some(f) => {
            let rb = HttpResponse::Ok().content_type("application/json");
            #[cfg(feature = "dashmap")] let response = rb.body(&f.value);
            #[cfg(feature = "cht")] let response = rb.body(f);
            response
        }
        None => {
            #[cfg(feature = "logging")]
            info!(
                "Cache expired for bus_stop_id: {}! Fetching from LTA servers.",
                bus_stop
            );

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

/// Swisstable implementation is left separate as it gets really hard to read if its added to the function above
#[cfg(feature = "swisstable")]
pub async fn get_timings(
    bus_stop: web::Path<u32>,
    lru: web::Data<RwLock<SwissCache<u32, String>>>,
    client: web::Data<LTAClient>,
) -> JustBusResult {
    let bus_stop = bus_stop.into_inner();
    let lru_r = lru.read();
    let in_lru = lru_r.get(bus_stop);

    let res = match in_lru {
        Some(f) => HttpResponse::Ok().content_type("application/json").body(f),
        None => {
            #[cfg(feature = "logging")]
            info!(
                "Cache expired for bus_stop_id: {}! Fetching from LTA servers.",
                bus_stop
            );
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
