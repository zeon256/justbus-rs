#[cfg(not(target_env = "msvc"))]
extern crate jemallocator;

use actix_web::{web, App, HttpServer};
use lta::{prelude::*, r#async::lta_client::LTAClient};
use std::env::var;
use std::time::Duration;

mod cache;
mod errors;
mod routes;

use crate::routes::{dummy, get_timings};

#[cfg(feature = "cht")]
use cht_time::Cache as ChtCache;

#[cfg(feature = "hashbrown")]
use hashbrown_time::Cache as HashBrownCache;

use parking_lot::RwLock;

#[cfg(feature = "vec")]
use vec_time::CacheVec;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let server_ip_and_port = var("IP_ADDR").unwrap_or("127.0.0.1:8080".to_string());
    println!("Starting server @ {}", &server_ip_and_port);

    let api_key = var("API_KEY").expect("API_KEY NOT FOUND!");
    let ttl = Duration::from_secs(15);
    let client = LTAClient::with_api_key(api_key);
    HttpServer::new(move || {
        let app = App::new()
            .route("/api/v1/dummy", web::get().to(dummy))
            .route("/api/v1/timings/{bus_stop}", web::get().to(get_timings))
            .data(client.clone());

        #[cfg(feature = "cht")]
        let app = app.data(ChtCache::<u32, String>::with_ttl_and_size(ttl, 500));

        #[cfg(feature = "hashbrown")]
        let app = app.data(RwLock::new(
            HashBrownCache::<u32, String>::with_ttl_and_size(ttl, 500),
        ));

        #[cfg(feature = "vec")]
        let app = app.data(RwLock::new(CacheVec::with_ttl(ttl)));

        app
    })
    .bind(server_ip_and_port)?
    .run()
    .await
}
