FROM ubuntu:20.04
ADD influxproxy /influxproxy
ENTRYPOINT ["/influxproxy"]