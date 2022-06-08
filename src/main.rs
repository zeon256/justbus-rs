use actix_web::{web, App, HttpServer};
use argh::FromArgs;
use lta::{Client, LTAClient};
use routes::bus_arrivals_bincode::bus_arrivals_bincode;
use std::{io, time::Duration};

mod errors;
mod routes;

use crate::routes::{bus_arrivals::*, health};

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
use rustls_pemfile::{certs, rsa_private_keys};

#[cfg(feature = "rustls")]
use rustls::{Certificate, PrivateKey, ServerConfig};

#[cfg(feature = "mimalloc")]
#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[cfg(feature = "jemalloc")]
#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static ALLOC: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

const TTL: Duration = Duration::from_secs(15);

// const DAY_TTL: Duration = Duration::from_secs(86400);
const SZ: usize = 5000;

#[cfg(feature = "tls")]
fn load_ssl_keys() -> ServerConfig {
    let cert_file = &mut BufReader::new(File::open("cert.pem").unwrap());
    let key_file = &mut BufReader::new(File::open("key-rsa.pem").unwrap());
    let cert_chain = certs(cert_file)
        .unwrap()
        .into_iter()
        .map(|s| Certificate(s))
        .collect();

    let mut keys = rsa_private_keys(key_file)
        .unwrap()
        .into_iter()
        .map(|s| PrivateKey(s))
        .collect::<Vec<_>>();

    ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(cert_chain, keys.remove(0))
        .unwrap()
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    let args: Args = argh::from_env();

    #[cfg(feature = "logging")]
    {
        std::env::set_var("RUST_LOG", "info, error");
        env_logger::init();
    }

    let ip_and_port = args.ip_addr.unwrap_or("127.0.0.1:8080".to_string());

    // things to note
    // if cache is created outside closure => global
    // else it is local to the thread worker

    #[cfg(all(feature = "dashmap", not(feature = "fxhash")))]
    let cache = web::Data::new(Cache::<u32, String>::with_ttl_and_size(TTL, SZ));

    #[cfg(all(feature = "dashmap", not(feature = "fxhash")))]
    let cache_bincode = web::Data::new(Cache::<u32, Vec<u8>>::with_ttl_and_size(TTL, SZ));

    #[cfg(all(feature = "swisstable", not(feature = "fxhash")))]
    let cache = web::Data::new(RwLock::new(SwissCache::<u32, String>::with_ttl_and_size(
        TTL, SZ,
    )));

    #[cfg(all(feature = "swisstable", not(feature = "fxhash")))]
    let cache_bincode = web::Data::new(RwLock::new(SwissCache::<u32, Vec<u8>>::with_ttl_and_size(
        TTL, SZ,
    )));

    #[cfg(all(feature = "fxhash", feature = "dashmap"))]
    let cache = {
        use std::hash::BuildHasherDefault;

        let hasher = BuildHasherDefault::<rustc_hash::FxHasher>::default();
        web::Data::new(Cache::<u32, String, _>::with_ttl_sz_and_hasher(
            TTL, SZ, hasher,
        ))
    };

    #[cfg(all(feature = "fxhash", feature = "dashmap"))]
    let cache_bincode = {
        use std::hash::BuildHasherDefault;

        let hasher = BuildHasherDefault::<rustc_hash::FxHasher>::default();
        web::Data::new(Cache::<u32, Vec<u8>, _>::with_ttl_sz_and_hasher(
            TTL, SZ, hasher,
        ))
    };

    #[cfg(all(feature = "fxhash", feature = "swisstable"))]
    let cache = {
        use std::hash::BuildHasherDefault;

        let hasher = BuildHasherDefault::<rustc_hash::FxHasher>::default();
        web::Data::new(RwLock::new(
            SwissCache::<u32, String, _>::with_ttl_sz_and_hasher(TTL, SZ, hasher),
        ))
    };

    #[cfg(all(feature = "fxhash", feature = "swisstable"))]
    let cache_bincode = {
        use std::hash::BuildHasherDefault;

        let hasher = BuildHasherDefault::<rustc_hash::FxHasher>::default();
        web::Data::new(RwLock::new(
            SwissCache::<u32, Vec<u8>, _>::with_ttl_sz_and_hasher(TTL, SZ, hasher),
        ))
    };

    let client = web::Data::new(LTAClient::with_api_key(args.api_key).unwrap());

    let server = HttpServer::new(move || {
        let app = App::new()
            .route("/api/v1/health", web::get().to(health))
            .route("/api/v1/timings/{bus_stop}", web::get().to(bus_arrivals))
            .route(
                "/api/v1/timings/bc/{bus_stop}",
                web::get().to(bus_arrivals_bincode),
            )
            .app_data(client.clone())
            .app_data(cache.clone())
            .app_data(cache_bincode.clone());

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

#[derive(FromArgs)]
/// Lightning fast API for Singapore LTA's bus data arrival timings with emphasis on low memory overhead, high throughput and low latency
struct Args {
    /// IP address of server. Defaults to 127.0.0.1:8080 if nothing is provided. eg.0.0.0.0:8080
    #[argh(option)]
    ip_addr: Option<String>,

    /// your LTA API key
    #[argh(option)]
    api_key: String,
}
