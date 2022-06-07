#[cfg(feature = "dashmap")]
use dashmap_time::Cache;

#[cfg(not(feature = "dashmap"))]
use hashbrown_time::Cache;

#[cfg(not(feature = "dashmap"))]
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::routes::JustBusResult;
use actix_web::{web, HttpResponse};

#[cfg(feature = "logging")]
use log::info;
use lta::{
    models::{
        bus::bus_arrival::{ArrivalBusService, NextBus},
        bus_enums::{BusFeature, BusLoad, BusType, Operator},
        chrono::{DateTime, FixedOffset},
    },
    Bus, BusRequests, LTAClient,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct ArrivalBusServiceBc {
    pub service_no: String,
    pub operator: Operator,
    pub next_bus: [Option<NextBusBc>; 3],
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct NextBusBc {
    pub origin_code: u32,
    pub dest_code: u32,
    pub est_arrival: DateTime<FixedOffset>,
    pub lat: f64,
    pub long: f64,
    pub visit_no: u32,
    pub load: BusLoad,
    pub feature: Option<BusFeature>,
    pub bus_type: BusType,
}

impl From<NextBus> for NextBusBc {
    fn from(s: NextBus) -> Self {
        Self {
            origin_code: s.origin_code,
            dest_code: s.dest_code,
            est_arrival: s.est_arrival,
            lat: s.lat,
            long: s.long,
            visit_no: s.visit_no,
            load: s.load,
            feature: s.feature,
            bus_type: s.bus_type,
        }
    }
}

impl From<ArrivalBusService> for ArrivalBusServiceBc {
    fn from(s: ArrivalBusService) -> Self {
        let [a, b, c] = s.next_bus;

        let next_bus = [
            a.map(NextBusBc::from),
            b.map(NextBusBc::from),
            c.map(NextBusBc::from),
        ];

        Self {
            service_no: s.service_no,
            operator: s.operator,
            next_bus,
        }
    }
}

pub async fn bus_arrivals_bincode(
    bus_stop: web::Path<u32>,
    #[cfg(feature = "dashmap")] lru: web::Data<Cache<u32, Vec<u8>>>,
    #[cfg(not(feature = "dashmap"))] lru: web::Data<RwLock<Cache<u32, Vec<u8>>>>,
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
        Some(f) => HttpResponse::Ok()
            .content_type("application/octet-stream")
            .body(f),
        None => {
            #[cfg(not(feature = "dashmap"))]
            drop(lru_r);

            #[cfg(feature = "logging")]
            info!(
                "bus_stop_id expired: {}! Fetching from LTA servers.",
                bus_stop
            );

            let arrivals = Bus::get_arrival(&client, bus_stop, None)
                .await?
                .services
                .into_iter()
                .map(ArrivalBusServiceBc::from)
                .collect::<Vec<_>>();

            let arrival_buf = bincode::serialize(&arrivals).unwrap();
            println!("{:?}", arrival_buf);

            #[cfg(feature = "dashmap")]
            lru.insert(bus_stop, arrival_buf.clone());

            #[cfg(not(feature = "dashmap"))]
            {
                let mut lru_w = lru.write();
                lru_w.insert(bus_stop, arrival_buf.clone());
            }

            HttpResponse::Ok()
                .content_type("application/octet-stream")
                .body(arrival_buf)
        }
    };

    Ok(res)
}

mod test {
    use crate::routes::bus_arrivals_bincode::ArrivalBusServiceBc;

    #[test]
    fn decode_test() {
        use lta::models::bus::bus_arrival::BusArrivalResp;
        use std::fs::File;
        use std::io::Read;

        let mut buf = vec![];
        let mut f = File::open("bincode_test_decode").unwrap();
        let _ = f.read_to_end(&mut buf);
        println!("{:?}", buf);

        let de = bincode::deserialize::<Vec<ArrivalBusServiceBc>>(&buf[..]).unwrap();
        dbg!(de);
    }
}
