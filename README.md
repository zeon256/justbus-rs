<p align="center">
  <img width="945" height="432" src="./logo.png">
    <a href="https://github.com/BudiNverse/justbus-rs">
      <img src="https://img.shields.io/badge/-justbus--rs-blueviolet.svg"/>
    </a>
    <a href="https://github.com/BudiNverse/justbus-rs">
        <img src="https://img.shields.io/github/license/BudiNverse/lta-rs"/>
    </a>
    <a href="https://dev.azure.com/budisyahiddin/lta-rs/_build?definitionId=7">
        <img src="https://dev.azure.com/budisyahiddin/lta-rs/_apis/build/status/BudiNverse.justbus-rs?branchName=master">  
    </a>
    <a href="https://github.com/BudiNverse/lta-rs">
        <img src="https://img.shields.io/badge/rust-1.3.9-blueviolet.svg"/>
    </a>      
</p>

# justbus-rs

>justbus-rs is a lightweight backend that serves LTA Datamall bus timings.
This project uses [lta-rs](https://github.com/BudiNverse/lta-rs) internally.

## Usage
```
GET http://localhost:8080/api/v1/timings/83139
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

## Optimisations (in order of impact)
- caching response with a lock-free hashmap
- caching only serialised data (ie `String`) to prevent tranforming struct to json response for every request
- `jemalloc`

## Performance
Disclaimer: benchmarks are naive and YMMV

i7 3770k @ 4.4Ghz 16G ram @ 2200Mhz, ubuntu 18.04 LTS  `wrk`
```
./wrk -c100 -d15s -t4 http://localhost:8080/api/v1/timings/83139 
```
```
zeon@zeon-desktop  ~  wrk -c100 -d15s -t4 http://localhost:8080/api/v1/timings/83139
Running 15s test @ http://localhost:8080/api/v1/timings/83139
  4 threads and 100 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     4.10ms   27.83ms 839.99ms   99.60%
    Req/Sec    64.04k    17.90k   89.25k    46.88%
  3812462 requests in 15.09s, 6.37GB read
  Non-2xx or 3xx responses: 115
Requests/sec: 252570.08
Transfer/sec:    431.87MB
```

Hello World benchmark
```
zeon@zeon-desktop  ~  wrk -c100 -d15s -t4 http://localhost:8080/api/v1/dummy
Running 15s test @ http://localhost:8080/api/v1/dummy
  4 threads and 100 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     1.33ms    2.74ms  38.44ms   89.56%
    Req/Sec    61.03k    15.00k   92.95k    61.47%
  3643319 requests in 15.10s, 444.74MB read
Requests/sec: 241334.14
Transfer/sec:     29.46MB
```

## How to build
Requirements: `jemalloc` and `libssl`
```
cargo build --release
```

## How to run
```
export API_KEY=YOUR_API_KEY
cargo run --release
```

## Docker
```
docker pull inverse/justbus_rs
docker run -d -p 8080:8080 -e API_KEY=YOUR_API_KEY -e IP_ADDR='0.0.0.0:8080' inverse/justbus_rs
```

## License
justbus-rs is licensed under MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)