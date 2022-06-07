<p align="center">
  <img width="945" height="432" src="./logo.png">
    <a href="https://github.com/BudiNverse/justbus-rs">
      <img src="https://img.shields.io/badge/-justbus--rs-blueviolet.svg"/>
    </a>
        <a href="https://github.com/BudiNverse/justbus-rs">
          <img src="https://img.shields.io/badge/version-0.3.0-ff69b4"/>
        </a>
    <a href="https://github.com/BudiNverse/justbus-rs">
        <img src="https://img.shields.io/github/license/BudiNverse/lta-rs"/>
    </a>
    <a href="">
        <img src="https://img.shields.io/github/workflow/status/BudiNverse/justbus-rs/Rust?logo=github">
    </a>
    <a href="https://github.com/BudiNverse/lta-rs">
        <img src="https://img.shields.io/badge/rust-1.5.6-blueviolet.svg?logo=rust"/>
    </a>      
</p>

# justbus-rs

>justbus-rs is a lightweight backend that serves LTA Datamall bus arrival timings with a strong emphasis on low memory usage, high throughput and low latency.
This project uses [lta-rs](https://github.com/BudiNverse/lta-rs) internally.

## Usage
Bus Arrival Timings
```
GET http://localhost:8080/api/v1/timings/<bus_stop_no>
```
<details>
<summary>
Click to show API response
</summary>

```json
[
    {
        "service_no": "15",
        "operator": "GAS",
        "next_bus": [
            {
                "origin_code": 77009,
                "dest_code": 77009,
                "est_arrival": "2020-01-04T15:12:31+08:00",
                "lat": 1.3254953333333335,
                "long": 103.90585966666667,
                "visit_no": 1,
                "load": "SeatsAvailable",
                "feature": "WheelChairAccessible",
                "bus_type": "SingleDecker"
            },
            {
                "origin_code": 77009,
                "dest_code": 77009,
                "est_arrival": "2020-01-04T15:19:03+08:00",
                "lat": 1.3351438333333334,
                "long": 103.9091055,
                "visit_no": 1,
                "load": "SeatsAvailable",
                "feature": "WheelChairAccessible",
                "bus_type": "SingleDecker"
            },
            {
                "origin_code": 77009,
                "dest_code": 77009,
                "est_arrival": "2020-01-04T15:33:05+08:00",
                "lat": 1.3459406666666667,
                "long": 103.9426515,
                "visit_no": 1,
                "load": "SeatsAvailable",
                "feature": "WheelChairAccessible",
                "bus_type": "SingleDecker"
            }
        ]
    },
    {
        "service_no": "150",
        "operator": "SBST",
        "next_bus": [
            {
                "origin_code": 82009,
                "dest_code": 82009,
                "est_arrival": "2020-01-04T15:07:50+08:00",
                "lat": 1.3147168333333332,
                "long": 103.90623166666667,
                "visit_no": 1,
                "load": "SeatsAvailable",
                "feature": "WheelChairAccessible",
                "bus_type": "SingleDecker"
            },
            {
                "origin_code": 82009,
                "dest_code": 82009,
                "est_arrival": "2020-01-04T15:20:54+08:00",
                "lat": 0.0,
                "long": 0.0,
                "visit_no": 1,
                "load": "SeatsAvailable",
                "feature": "WheelChairAccessible",
                "bus_type": "SingleDecker"
            },
            {
                "origin_code": 82009,
                "dest_code": 82009,
                "est_arrival": "2020-01-04T15:32:54+08:00",
                "lat": 0.0,
                "long": 0.0,
                "visit_no": 1,
                "load": "SeatsAvailable",
                "feature": "WheelChairAccessible",
                "bus_type": "SingleDecker"
            }
        ]
    },
    {
        "service_no": "155",
        "operator": "SBST",
        "next_bus": [
            {
                "origin_code": 52009,
                "dest_code": 84009,
                "est_arrival": "2020-01-04T15:07:39+08:00",
                "lat": 1.31445,
                "long": 103.90634883333334,
                "visit_no": 1,
                "load": "StandingAvailable",
                "feature": "WheelChairAccessible",
                "bus_type": "SingleDecker"
            },
            {
                "origin_code": 52009,
                "dest_code": 84009,
                "est_arrival": "2020-01-04T15:24:52+08:00",
                "lat": 1.3201654999999999,
                "long": 103.88181566666667,
                "visit_no": 1,
                "load": "SeatsAvailable",
                "feature": "WheelChairAccessible",
                "bus_type": "SingleDecker"
            },
            {
                "origin_code": 52009,
                "dest_code": 84009,
                "est_arrival": "2020-01-04T15:42:38+08:00",
                "lat": 1.3347515,
                "long": 103.8782805,
                "visit_no": 1,
                "load": "StandingAvailable",
                "feature": "WheelChairAccessible",
                "bus_type": "SingleDecker"
            }
        ]
    }
]
```

</details>

## Feature flags
The following features can be activated during compile time. Program will **NOT** compile if there are no feature flags! Multiple caching strategies are implemented as 
different machines perform differently with each of them. A machine with lesser physical cores may benefit more from `dashmap` whereas a higher core machine may not see any difference between any of the caching strategies. As always, you should benchmark them yourself if performance is a concern!
- `swisstable` (recommended)
- [`dashmap`](https://github.com/xacrimon/dashmap)
- `tls` (using Rustls)
- `logging` 
- `hw-lock-elision` (To be paired with `swisstable`, enables hardware lock elision for `RwLock`)

## Optimisations (in order of impact)
- Caching response with a hashmap
- Amortised serialisation of data structures
- `jemalloc`

## Performance
Disclaimer: benchmarks are naive and YMMV

AMD Ryzen 3600 @ 4.3Ghz (stock) 16G ram @ 3600Mhz, ubuntu 20.04 LTS
```
wrk -c100 -d15s -t6 http://localhost:8080/api/v1/timings/83139 
```

### Swisstable **(Recommended)**
```
Running 15s test @ http://localhost:8080/api/v1/timings/83139
  6 threads and 100 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     1.40ms    4.49ms 109.89ms   92.99%
    Req/Sec   126.60k    33.29k  172.64k    74.11%
  11357888 requests in 15.05s, 23.48GB read
  Non-2xx or 3xx responses: 59
Requests/sec:   754475.17
Transfer/sec:      1.56GB

Memory Usage @ Peak: 21MB
```

## How to build
Requirements: `jemalloc` and `libssl`. Binary will be at `/target/release` folder.
```
# Lets say we want to use dashmap, logging and tls
cargo build --release --features tls,logging,dashmap
```

## How to run
```
cd ./target/release
./justbus-rs --ip-addr IP_ADDR_YOU_WANT_WITH_PORT --api-key YOUR_API_KEY

# Do note that ip-addr is optional and will default to 127.0.0.1:8080 if nothing is provided
# api-key is a must
```

## TLS Guide
We put the self-signed certificate in this directory as an example but your browser would complain that it isn't secure. So we recommend to use `mkcert` to trust it. To use local CA, you should run:
```
mkcert -install
```

If you want to generate your own cert/private key file, then run:
```
mkcert localhost
openssl rsa -in localhost-key.pem -out key-rsa.pem

# Then run the program normally
```

## Docker
By default, docker image uses `swisstable` feature. If you need the other features, you will need to modify Dockerfile and build your own image.
```
docker pull inverse/justbus_rs
docker run -d -p 8080:8080 inverse/justbus_rs --api-key YOUR_API_KEY --ip-addr 0.0.0.0:8080
```

## Contributors
Do send a PR if you think you have improvements to make whether to the actual codebase or any of the documentation!

## License
justbus-rs is licensed under MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)