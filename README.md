# justbus-rs

>justbus-rs is a lightweight backend that serves LTA Datamall bus timings.
This project uses [lta-rs](https://github.com/BudiNverse/lta-rs) internally.

## Usage
```
GET http://localhost:8080/api/v1/timings/83139
```
```json
{
    "BusStopCode": 83139,
    "Data": [
        {
            "ServiceNo": "15",
            "Operator": "GAS",
            "NextBus": {
                "OriginCode": 77009,
                "DestinationCode": 77009,
                "EstimatedArrival": "2019-09-28T10:49:24+08:00",
                "Latitude": 1.3156283333333334,
                "Longitude": 103.90589216666666,
                "VisitNumber": 1,
                "Load": "SDA",
                "Feature": "WAB",
                "Type": "SD"
            },
            "NextBus2": {
                "OriginCode": 77009,
                "DestinationCode": 77009,
                "EstimatedArrival": "2019-09-28T11:03:49+08:00",
                "Latitude": 1.3382883333333333,
                "Longitude": 103.91794783333333,
                "VisitNumber": 1,
                "Load": "SEA",
                "Feature": "WAB",
                "Type": "SD"
            },
            "NextBus3": {
                "OriginCode": 77009,
                "DestinationCode": 77009,
                "EstimatedArrival": "2019-09-28T11:12:23+08:00",
                "Latitude": 1.3450285,
                "Longitude": 103.93672483333333,
                "VisitNumber": 1,
                "Load": "SEA",
                "Feature": "WAB",
                "Type": "SD"
            }
        },
        {
            "ServiceNo": "150",
            "Operator": "SBST",
            "NextBus": {
                "OriginCode": 82009,
                "DestinationCode": 82009,
                "EstimatedArrival": "2019-09-28T10:55:40+08:00",
                "Latitude": 0,
                "Longitude": 0,
                "VisitNumber": 1,
                "Load": "SEA",
                "Feature": "WAB",
                "Type": "SD"
            },
            "NextBus2": {
                "OriginCode": 82009,
                "DestinationCode": 82009,
                "EstimatedArrival": "2019-09-28T11:23:00+08:00",
                "Latitude": 0,
                "Longitude": 0,
                "VisitNumber": 1,
                "Load": "SEA",
                "Feature": "WAB",
                "Type": "SD"
            },
            "NextBus3": null
        },
        {
            "ServiceNo": "155",
            "Operator": "SBST",
            "NextBus": {
                "OriginCode": 52009,
                "DestinationCode": 84009,
                "EstimatedArrival": "2019-09-28T11:03:13+08:00",
                "Latitude": 1.3189968333333333,
                "Longitude": 103.88656283333333,
                "VisitNumber": 1,
                "Load": "SEA",
                "Feature": "WAB",
                "Type": "SD"
            },
            "NextBus2": {
                "OriginCode": 52009,
                "DestinationCode": 84009,
                "EstimatedArrival": "2019-09-28T11:19:25+08:00",
                "Latitude": 1.3275061666666668,
                "Longitude": 103.88342483333334,
                "VisitNumber": 1,
                "Load": "SEA",
                "Feature": "WAB",
                "Type": "SD"
            },
            "NextBus3": null
        }
    ]
}
```

### Versions
There are 3 different versions of this project.
- `actor` style
- `standard` style
- `lazy_static` global

## Performance
Benchmarks here is very simplistic and probably is not scientific in any way.

All benchmark conducted on i7 3770k @ 4.4Ghz 16G ram @ 2200Mhz, linux, 8 threads using `wrk`
```
./wrk -c100 -d15s -t8 http://localhost:8080/api/v1/timings/83139 
```

### `actor` style
Uses actix actors
```
Running 15s test @ http://localhost:8080/api/v1/timings/83139
  8 threads and 100 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     3.25ms    4.72ms 130.32ms   91.45%
    Req/Sec     4.91k   423.14     7.20k    80.08%
  586656 requests in 15.04s, 1.05GB read
  Non-2xx or 3xx responses: 53
Requests/sec:  39012.75
Transfer/sec:     71.73MB
```
Memory usage
```
Initial memory usage: 1.5MiB
Idle memory usage after 1 round: 9.2MiB
Benchmark memory usage: 13.6MiB
```

### `standard` style
Uses RwLock
```
Running 15s test @ http://localhost:8080/api/v1/timings/83139
  8 threads and 100 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     1.75ms    4.65ms 126.20ms   93.37%
    Req/Sec    17.05k     4.24k   31.19k    74.52%
  2046115 requests in 15.10s, 4.01GB read
  Non-2xx or 3xx responses: 52
Requests/sec: 135476.59
Transfer/sec:    271.96MB
```
Memory usage
```
Initial memory usage: 3.5MiB
Idle memory usage after 1 round: 13.7MiB
Benchmark memory usage: 13.7MiB
```

### `lazy_static` style
Uses RwLock and lazy_static
```
Running 15s test @ http://localhost:8080/api/v1/timings/83139
  8 threads and 100 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     1.45ms    4.34ms 134.37ms   96.36%
    Req/Sec    16.55k     4.46k   73.19k    73.58%
  1981869 requests in 15.10s, 4.00GB read
  Non-2xx or 3xx responses: 53
Requests/sec: 131223.26
Transfer/sec:    271.18MB
```
Memory usage
```
Initial memory usage: 2.6MiB
Idle memory usage after 1 round: 13.9MiB
Benchmark memory usage: 13.9MiB
```