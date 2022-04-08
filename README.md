# Influxproxy

Influxproxy is a proxy for [InfluxDB].(https://github.com/influxdata/influxdb).

## Installation

docker:
```
docker run --rm --network host --name influxproxy -e "INFLUXPROXY_TOKEN=TOKEN_OF_THE_INFLUXDB" easybill/influxproxy:v0.0.9
```

download:
- [influxproxy_linux_latest_x86_64](https://github.com/easybill/influxproxy/releases/latest/download/influxproxy_ubuntu-latest_x86_64)
- [influxproxy_linux_latest_aarch64](https://github.com/easybill/influxproxy/releases/latest/download/influxproxy_ubuntu-latest_aarch64)
- [influxproxy_mac_latest_aarch64](https://github.com/easybill/influxproxy/releases/latest/download/influxproxy_mac_aarch64)
- [influxproxy_mac_x86_64](https://github.com/easybill/influxproxy/releases/latest/download/influxproxy_mac_x86_64)

#### php api

```
# currently it is not pushed to packagist, so add this repository to your composer.json
"repositories": [
    {"type": "vcs", "url": "https://github.com/easybill/influxproxy"}
[

# and add to composer package
"require": {
    "easybill/influxproxy": "dev-main"
}
```

## How Influxproxy Works

Influxproxy provides only one endpoint for [Write Data](https://docs.influxdata.com/influxdb/v2.2/api/#operation/PostWrite)
and is designed to receive data as quickly as possible and pass it to InfluxDB.
The API is not fully compatible, but very similar.

We install Influxproxy on all systems that generate a lot of data points, but cannot
intelligently buffer them themselves.
For example, if you want to write some data points to InfluxDB on a web page call,
you probably want to avoid making N HTTP calls against a foreign system every time.
Instead, you can send the data points to Influxproxy, here the data is aggregated and passed on.
The application only talks to the extremely fast Influxproxy.
Because Influxproxy runs locally, there is virtually no latency and even (hundreds of) thousands of requests are completely unproblematic.
The InfluxDB is also protected, since the data points are pre-aggregated.
If the InfluxDB is not accessible, it is simply tried again later.