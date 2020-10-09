#[cfg(not(target_env = "msvc"))]
extern crate jemallocator;

use actix_web::{web, App, HttpServer};
use lta::{prelude::*, r#async::lta_client::LTAClient};
use std::{env, io, time::Duration};

mod errors;
mod routes;

use crate::routes::{get_timings, health};

#[cfg(feature = "cht")]
use cht_time::Cache;

#[cfg(feature = "swisstable")]
use hashbrown_time::Cache as SwissCache;

#[cfg(feature = "swisstable")]
use parking_lot::RwLock;

#[cfg(feature = "dashmap")]
use dashmap_time::Cache;

#[cfg(feature = "logging")]
use actix_web::middleware::Logger;

#[cfg(feature = "tls")]
use std::{fs::File, io::BufReader};

#[cfg(feature = "tls")]
use rustls::{
    internal::pemfile::{certs, rsa_private_keys},
    NoClientAuth, ServerConfig,
};

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

const TTL: Duration = Duration::from_secs(15);
const SZ: usize = 500;

#[cfg(feature = "tls")]
fn load_ssl_keys() -> ServerConfig {
    let mut config = ServerConfig::new(NoClientAuth::new());
    let cert_file = &mut BufReader::new(File::open("cert.pem").unwrap());
    let key_file = &mut BufReader::new(File::open("key.pem").unwrap());
    let cert_chain = certs(cert_file).unwrap();
    let mut keys = rsa_private_keys(key_file).unwrap();
    config.set_single_cert(cert_chain, keys.remove(0)).unwrap();

    config
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    #[cfg(feature = "logging")]
    {
        env::set_var("RUST_LOG", "info, error");
        env_logger::init();
    }

    let ip_and_port = env::var("IP_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".to_string());

    let api_key = env::var("API_KEY").expect("API_KEY NOT FOUND!");
    let client = LTAClient::with_api_key(api_key);
    let server = HttpServer::new(move || {
        let app = App::new()
            .route("/api/v1/health", web::get().to(health))
            .route("/api/v1/timings/{bus_stop}", web::get().to(get_timings))
            .data(client.clone());

        #[cfg(any(feature = "cht", feature = "dashmap"))]
        let app = app.data(Cache::<u32, String>::with_ttl_and_size(TTL, SZ));

        #[cfg(feature = "swisstable")]
        let app = app.data(RwLock::new(SwissCache::<u32, String>::with_ttl_and_size(
            TTL, SZ,
        )));

        #[cfg(feature = "logging")]
        let app = app
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"));

        app
    });

    #[cfg(feature = "tls")]
    let res = {
        let config = load_ssl_keys();
        server.bind_rustls(ip_and_port, config)?.run().await
    };

    #[cfg(not(feature = "tls"))]
    let res = server.bind(ip_and_port)?.run().await;

    res
}
