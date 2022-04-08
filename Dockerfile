FROM ubuntu:20.04
ADD influxproxy /influxproxy
RUN chmod +x /influxproxy
ENTRYPOINT ["/influxproxy"]