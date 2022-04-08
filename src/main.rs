use axum::body::Bytes;
use axum::extract::{Extension, Path};
use axum::{routing::post, Json, Router};

use serde_json::{json, Value};
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use std::time::Duration;
use structopt::StructOpt;

#[derive(StructOpt, Debug, Clone)]
#[structopt(name = "basic")]
pub struct Opt {
    #[structopt(env = "INFLUXPROXY_TOKEN")]
    token: String,
    #[structopt(long = "bind", default_value = "0.0.0.0:3343", env = "INFLUXPROXY_BIND")]
    bind: String,
    #[structopt(long = "influx_endpoint", default_value = "http://127.0.0.1:8086", env = "INFLUXPROXY_ENDPOINT")]
    influx_endpoint: String,
}

#[derive(Debug)]
pub struct Metric {
    data: String,
    org: String,
    bucket: String,
}

#[derive(Debug)]
pub struct AggregatedMetric {
    org: String,
    bucket: String,
    data: Vec<String>,
}

#[tokio::main]
async fn main() -> ::anyhow::Result<(), ::anyhow::Error> {
    let opt: Opt = Opt::from_args();
    tracing_subscriber::fmt::init();

    println!("{:?}", opt);

    let (sender, receiver) = ::tokio::sync::mpsc::unbounded_channel::<Metric>();
    let (agg_sender, agg_receiver) = ::tokio::sync::mpsc::unbounded_channel::<AggregatedMetric>();

    tokio::spawn(async move { aggregate_metric_data(receiver, agg_sender).await });

    let opt2 = opt.clone();
    tokio::spawn(async move { send_data_to_influx(agg_receiver, opt2).await });

    let app = Router::new()
        .route("/write/:org/:bucket", post(handler_write))
        .layer(Extension(sender));

    let addr = opt.bind.parse::<SocketAddr>().expect("");
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn handler_write(
    Extension(sender): Extension<UnboundedSender<Metric>>,
    Path((org, bucket)): Path<(String, String)>,
    body: Bytes,
) -> Json<Value> {
    let data = extract_data(bucket, org, String::from_utf8_lossy(&body[..]).to_string());
    dbg!(&data);
    for metric in data {
        match sender.send(metric) {
            Ok(()) => {}
            Err(err) => println!("{}", &err),
        };
    }
    Json(json!({
        "success": true,
    }))
}

fn extract_data(bucket: String, org: String, body: String) -> Vec<Metric> {
    body.split("\n")
        .map(|v| v.trim())
        .map(|v| Metric {
            data: v.to_string(),
            bucket: bucket.clone(),
            org: org.clone(),
        })
        .collect()
}

async fn aggregate_metric_data(
    mut receiver: UnboundedReceiver<Metric>,
    sender: UnboundedSender<AggregatedMetric>,
) -> Result<(), ::anyhow::Error> {
    let mut metrics: HashMap<(String, String), Vec<String>> = HashMap::new();
    loop {
        let sleep = tokio::time::sleep(Duration::from_secs(5));
        tokio::select! {
            _ = sleep => {
                let mut metric_chunk: HashMap<(String, String), Vec<String>> = HashMap::new();
                ::std::mem::swap(&mut metrics, &mut metric_chunk);

                for (key, value) in metric_chunk.into_iter() {
                    sender.send(AggregatedMetric { org: key.1, bucket: key.0, data: value })?;
                }
            }
            raw_entry = receiver.recv() => {
                let metric = match raw_entry {
                    Some(metric) => metric,
                    None => continue
                };
                metrics.entry((metric.bucket, metric.org))
                    .or_insert(vec![])
                    .push(metric.data);
            }
        }
    }
}

async fn send_data_to_influx(
    mut receiver: UnboundedReceiver<AggregatedMetric>,
    opt: Opt,
) -> Result<(), ::anyhow::Error> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .connect_timeout(Duration::from_secs(10))
        .build()?;
    loop {
        let metric = match receiver.recv().await {
            Some(s) => s,
            None => continue,
        };

        loop {
            match client
                .post(format!(
                    "{}/api/v2/write?bucket={}&org={}&precision=s",
                    opt.influx_endpoint, &metric.bucket, &metric.org
                ))
                .header("Authorization", format!("Token {}", opt.token))
                .body(metric.data.join("\n").clone())
                .send()
                .await
            {
                Ok(response) if response.status() == 204 => {
                    println!(
                        "Sent {} metrics to influx (org: {}, bucket: {})",
                        metric.data.len(),
                        metric.org,
                        metric.bucket
                    );
                    break;
                }
                Ok(response) => {
                    println!(
                        "Unexpected response from influx database! {:#?}",
                        response.bytes().await
                    );
                    ::tokio::time::sleep(Duration::from_secs(1)).await;
                    continue;
                }
                Err(err) => {
                    println!("Unable to send metric data: {}", err);
                    ::tokio::time::sleep(Duration::from_secs(1)).await;
                    continue;
                }
            }
        }
    }
}
