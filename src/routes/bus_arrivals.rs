#[cfg(feature = "dashmap")]
use dashmap_time::Cache;

#[cfg(not(feature = "dashmap"))]
use hashbrown_time::Cache;

#[cfg(not(feature = "dashmap"))]
use parking_lot::RwLock;

use crate::routes::JustBusResult;
use actix_web::{web, HttpResponse};

#[cfg(feature = "logging")]
use log::info;
use lta::{Bus, BusRequests, LTAClient};

pub async fn bus_arrivals(
    bus_stop: web::Path<u32>,
    #[cfg(feature = "dashmap")] lru: web::Data<Cache<u32, String>>,
    #[cfg(not(feature = "dashmap"))] lru: web::Data<RwLock<Cache<u32, String>>>,
    client: web::Data<LTAClient>,
) -> JustBusResult {
    let bus_stop = bus_stop.into_inner();

    #[cfg(not(feature = "dashmap"))]
    let lru_r = lru.read();

    #[cfg(feature = "dashmap")]
    let in_lru = lru.get(&bus_stop).map(|s| s.value.clone());

    #[cfg(not(feature = "dashmap"))]
    let in_lru = lru_r.get(bus_stop).cloned();

    let res = match in_lru {
        Some(f) => HttpResponse::Ok().content_type("application/json").body(f),
        None => {
            
            #[cfg(not(feature = "dashmap"))]
            drop(lru_r);

            #[cfg(feature = "logging")]
            info!(
                "bus_stop_id expired: {}! Fetching from LTA servers.",
                bus_stop
            );

            let arrivals = Bus::get_arrival(&client, bus_stop, None).await?;

            let arrival_str = serde_json::to_string(&arrivals.services).unwrap();

            #[cfg(feature = "dashmap")]
            lru.insert(bus_stop, arrival_str.clone());

            #[cfg(not(feature = "dashmap"))]
            {
                let mut lru_w = lru.write();
                lru_w.insert(bus_stop, arrival_str.clone());
            }

            HttpResponse::Ok()
                .content_type("application/json")
                .body(arrival_str)
        }
    };

    Ok(res)
}
