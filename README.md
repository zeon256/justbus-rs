<p align="center">
  <img width="945" height="432" src="./logo.png">
    <a href="https://github.com/BudiNverse/justbus-rs">
      <img src="https://img.shields.io/badge/-justbus--rs-blueviolet.svg"/>
    </a>
    <a href="https://github.com/BudiNverse/justbus-rs">
        <img src="https://img.shields.io/github/license/BudiNverse/justbus-rs"/>
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
```json
{
    "bus_stop_code": 83139,
    "services": [
        {
            "service_no": "15",
            "operator": "GAS",
            "next_bus": [
                {
                    "origin_code": 77009,
                    "dest_code": 77009,
                    "est_arrival": "2020-01-03T19:17:05+08:00",
                    "lat": 1.338279,
                    "long": 103.91795633333334,
                    "visit_no": 1,
                    "load": "SeatsAvailable",
                    "feature": "WheelChairAccessible",
                    "bus_type": "SingleDecker"
                },
                {
                    "origin_code": 77009,
                    "dest_code": 77009,
                    "est_arrival": "2020-01-03T19:30:48+08:00",
                    "lat": 1.3516848333333333,
                    "long": 103.94422416666667,
                    "visit_no": 1,
                    "load": "SeatsAvailable",
                    "feature": "WheelChairAccessible",
                    "bus_type": "SingleDecker"
                },
                null
            ]
        },
        {
            "service_no": "150",
            "operator": "SBST",
            "next_bus": [
                {
                    "origin_code": 82009,
                    "dest_code": 82009,
                    "est_arrival": "2020-01-03T18:57:35+08:00",
                    "lat": 1.3148378333333333,
                    "long": 103.9098375,
                    "visit_no": 1,
                    "load": "StandingAvailable",
                    "feature": "WheelChairAccessible",
                    "bus_type": "SingleDecker"
                },
                {
                    "origin_code": 82009,
                    "dest_code": 82009,
                    "est_arrival": "2020-01-03T19:11:12+08:00",
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
                    "est_arrival": "2020-01-03T19:23:12+08:00",
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
                    "est_arrival": "2020-01-03T19:11:50+08:00",
                    "lat": 1.3198983333333334,
                    "long": 103.8912305,
                    "visit_no": 1,
                    "load": "StandingAvailable",
                    "feature": "WheelChairAccessible",
                    "bus_type": "SingleDecker"
                },
                {
                    "origin_code": 52009,
                    "dest_code": 84009,
                    "est_arrival": "2020-01-03T19:26:45+08:00",
                    "lat": 1.3236286666666666,
                    "long": 103.88722233333333,
                    "visit_no": 1,
                    "load": "StandingAvailable",
                    "feature": "WheelChairAccessible",
                    "bus_type": "SingleDecker"
                },
                null
            ]
        }
    ]
}
```

## Optimisations (in order of impact)
- caching response with a lock-free hashmap
- caching only serialised data (ie `String`) to prevent tranforming struct to json response for every request
- jemalloc

## Performance
Disclaimer: benchmarks are naive and YMMV

All benchmark conducted on i7 3770k @ 4.4Ghz 16G ram @ 2200Mhz, ubuntu 18.04 LTS  `wrk`
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