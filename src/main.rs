#[cfg(not(target_env = "msvc"))]
extern crate jemallocator;

use actix_web::{web, App, HttpServer};
use lta::{prelude::*, r#async::lta_client::LTAClient};
use std::time::Duration;
use std::{env, io};

mod errors;
mod routes;

use crate::routes::{dummy, get_timings};

#[cfg(feature = "cht")]
use cht_time::Cache as ChtCache;

#[cfg(feature = "hashbrown")]
use hashbrown_time::Cache as HashBrownCache;

#[cfg(feature = "dashmap_cache")]
use dashmap::DashMap;

use internal_entry::InternalEntry;
#[cfg(feature = "hashbrown")]
use parking_lot::RwLock;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

pub const TTL: Duration = Duration::from_secs(15);

#[actix_web::main]
async fn main() -> io::Result<()> {
    let ip_and_port = env::var("IP_ADDR").unwrap_or("127.0.0.1:8080".to_string());
    println!("Starting server @ {}", &ip_and_port);

    let api_key = env::var("API_KEY").expect("API_KEY NOT FOUND!");
    let client = LTAClient::with_api_key(api_key);
    HttpServer::new(move || {
        let app = App::new()
            .route("/api/v1/dummy", web::get().to(dummy))
            .route("/api/v1/timings/{bus_stop}", web::get().to(get_timings))
            .data(client.clone());

        #[cfg(feature = "cht")]
        let app = app.data(ChtCache::<u32, String>::with_ttl_and_size(TTL, 500));

        #[cfg(feature = "hashbrown")]
        let app = app.data(RwLock::new(
            HashBrownCache::<u32, String>::with_ttl_and_size(TTL, 500),
        ));

        #[cfg(feature = "dashmap_cache")]
        let app = app.data(DashMap::<u32, InternalEntry<String>>::with_capacity(500));

        app
    })
    .bind(ip_and_port)?
    .run()
    .await
}
