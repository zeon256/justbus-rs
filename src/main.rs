#[cfg(not(target_env = "msvc"))]
extern crate jemallocator;

#[cfg(target_os = "windows")]
use mimalloc::MiMalloc;

use actix_web::{web, App, HttpServer};
use argh::FromArgs;
use lta::{Client, LTAClient};
use std::{io, time::Duration};

#[cfg(feature = "logging")]
use std::env;

mod errors;
mod routes;

use crate::routes::{bus_arrivals::*, health};

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

#[cfg(target_os = "windows")]
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

const TTL: Duration = Duration::from_secs(15);

const SZ: usize = 500;

#[cfg(feature = "tls")]
fn load_ssl_keys() -> io::Result<rustls::ServerConfig> {
    use rustls::{NoClientAuth, ServerConfig, internal::pemfile::{certs, rsa_private_keys}};
    use std::{fs::File, io::BufReader};


    let cert = &mut BufReader::new(File::open("cert.pem")?);
    let key = &mut BufReader::new(File::open("key-rsa.pem")?);

    let certs = certs(cert).unwrap();
    let mut private_key = rsa_private_keys(key).unwrap();

    let mut server_cfg = ServerConfig::new(NoClientAuth::new());
    server_cfg
        .set_single_cert(certs, private_key.remove(0))
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Bad cert/key"))?;

    Ok(server_cfg)
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    let args: Args = argh::from_env();

    #[cfg(feature = "logging")]
    {
        env::set_var("RUST_LOG", "info, error");
        env_logger::init();
    }

    let ip_and_port = args.ip_addr.unwrap_or("127.0.0.1:8080".to_string());
    let client = LTAClient::with_api_key(args.api_key)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid API Key!"))?;

    #[cfg(any(feature = "cht", feature = "dashmap"))]
    let cache = web::Data::new(Cache::<u32, String>::with_ttl_and_size(TTL, SZ));

    #[cfg(feature = "swisstable")]
    let cache = web::Data::new(RwLock::new(SwissCache::<u32, String>::with_ttl_and_size(
        TTL, SZ,
    )));

    let server = HttpServer::new(move || {
        let app = App::new()
            .route("/api/v1/health", web::get().to(health))
            .route("/api/v1/timings/{bus_stop}", web::get().to(bus_arrivals))
            .app_data(web::Data::new(client.clone()));

        let app = app.app_data(cache.clone());

        #[cfg(feature = "logging")]
        let app = app
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"));

        app
    });

    #[cfg(feature = "tls")]
    let res = {
        let config = load_ssl_keys()?;
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
