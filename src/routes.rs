//! All functions are kept seperate for ease of reading
//! and to make it easier to add or remove stuff

use crate::errors::JustBusError;
use actix_web::{web, HttpResponse};
use lta::r#async::bus;
use lta::r#async::lta_client::LTAClient;

#[cfg(feature = "cht")]
use cht_time::Cache;

#[cfg(feature = "dashmap")]
use dashmap_time::Cache;

#[cfg(feature = "swisstable")]
use hashbrown_time::Cache as SwissCache;

#[cfg(feature = "swisstable")]
use parking_lot::RwLock;

#[cfg(feature = "logging")]
use log::info;

type JustBusResult = Result<HttpResponse, JustBusError>;

pub async fn health() -> &'static str {
    "hello_world"
}

#[cfg(any(feature = "cht", feature = "dashmap"))]
pub async fn bus_stops(
    lru: web::Data<Cache<u32, String>>,
    client: web::Data<LTAClient>,
) -> JustBusResult {
    unimplemented!()
}

#[cfg(any(feature = "cht", feature = "dashmap"))]
pub async fn bus_routes(
    lru: web::Data<Cache<u32, String>>,
    client: web::Data<LTAClient>,
) -> JustBusResult {
    unimplemented!()
}

/// Key is 0 as there is no `is_empty` implemented for Cache
#[cfg(any(feature = "cht", feature = "dashmap"))]
pub async fn bus_services(
    lru: web::Data<Cache<u8, String>>,
    client: web::Data<LTAClient>,
) -> JustBusResult {
    let in_lru = lru.get(&0);

    let res = match in_lru {
        #[rustfmt::skip]
        Some(f) => {
            #[cfg(feature = "dashmap")] let response = HttpResponse::Ok().content_type("application/json").body(&f.value);
            #[cfg(feature = "cht")] let response = HttpResponse::Ok().content_type("application/json").body(f);
            response
        }
        None => {
            #[cfg(feature = "logging")]
            info!("Cache expired for bus_services! Fetching from LTA servers.");
            let mut buf = vec![];

            let mut counter = 0;
            // Only 500 entries are returned so we have to query LTA servers to get all the services
            loop {
                let mut bus_services = bus::get_bus_services(&client, Some(counter)).await?;

                buf.append(&mut bus_services);
                counter += 500;
                if bus_services.is_empty() {
                    break;
                }
            }

            let bus_services_str = serde_json::to_string(&buf).unwrap();
            lru.insert(0, bus_services_str.clone());

            HttpResponse::Ok()
                .content_type("application/json")
                .body(bus_services_str)
        }
    };

    Ok(res)
}

#[cfg(any(feature = "cht", feature = "dashmap"))]
pub async fn bus_arrivals(
    bus_stop: web::Path<u32>,
    lru: web::Data<Cache<u32, String>>,
    client: web::Data<LTAClient>,
) -> JustBusResult {
    let bus_stop = bus_stop.into_inner();
    let in_lru = lru.get(&bus_stop);

    let res = match in_lru {
        #[rustfmt::skip]
        Some(f) => {
            #[cfg(feature = "dashmap")] let response = HttpResponse::Ok().content_type("application/json").body(&f.value);
            #[cfg(feature = "cht")] let response = HttpResponse::Ok().content_type("application/json").body(f);
            response
        }
        None => {
            #[cfg(feature = "logging")]
            info!(
                "Cache expired for bus_stop_id: {}! Fetching from LTA servers.",
                bus_stop
            );

            let arrivals = bus::get_arrival(&client, bus_stop, None).await?.services;

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
pub async fn bus_arrivals(
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

            let arrivals = bus::get_arrival(&client, bus_stop, None).await?.services;

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
