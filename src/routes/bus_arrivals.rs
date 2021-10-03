#[cfg(feature = "dashmap")]
use dashmap_time::Cache;

#[cfg(feature = "swisstable")]
use hashbrown_time::Cache as SwissCache;

#[cfg(feature = "swisstable")]
use parking_lot::RwLock;

use crate::routes::JustBusResult;
use actix_web::{web, HttpResponse};

#[cfg(feature = "logging")]
use log::info;

use lta::Bus;
use lta::{BusRequests, LTAClient};

#[cfg(feature = "dashmap")]
pub async fn bus_arrivals(
    bus_stop: web::Path<u32>,
    lru: web::Data<Cache<u32, String>>,
    client: web::Data<LTAClient>,
) -> JustBusResult {
    let bus_stop = bus_stop.into_inner();
    let in_lru = lru.get(&bus_stop);

    let res = match in_lru {
        Some(f) => HttpResponse::Ok().content_type("application/json").body(&f.value),
        None => {
            #[cfg(feature = "logging")]
            info!(
                "bus_stop_id expired: {}! Fetching from LTA servers.",
                bus_stop
            );

            let arrivals = Bus::get_arrival(&client, bus_stop, None).await?;

            let arrival_str = serde_json::to_string(&arrivals.services).unwrap();
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
            // drop the lock
            drop(lru_r);

            #[cfg(feature = "logging")]
            info!(
                "bus_stop_id expired: {}! Fetching from LTA servers.",
                bus_stop
            );

            let arrivals = Bus::get_arrival(&client, bus_stop, None).await?;

            let mut lru_w = lru.write();
            let arrival_str = serde_json::to_string(&arrivals.services).unwrap();
            lru_w.insert(bus_stop, arrival_str.clone());

            HttpResponse::Ok()
                .content_type("application/json")
                .body(arrival_str)
        }
    };

    Ok(res)
}
