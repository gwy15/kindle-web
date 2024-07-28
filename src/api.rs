use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::config::InfluxConfig;

#[derive(Debug, Serialize)]
pub struct Data<T = f32, H = f32> {
    pub temp: T,
    pub temp_time: DateTime<Utc>,
    pub humid: H,
    pub humid_time: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
struct Row<V> {
    _time: DateTime<Utc>,
    _value: V,
}

pub async fn get(client: &Client, config: &InfluxConfig) -> Result<Data> {
    let temp = get_inner::<f32>(&config.temp_query, client, config).await?;
    let humid = get_inner::<f32>(&config.humid_query, client, config).await?;
    Ok(Data {
        temp: temp._value,
        temp_time: temp._time,
        humid: humid._value,
        humid_time: humid._time,
    })
}

async fn get_inner<T: DeserializeOwned>(
    q: &str,
    client: &Client,
    config: &InfluxConfig,
) -> Result<Row<T>> {
    let content = client
        .post(format!(
            "https://{domain}/api/v2/query?org={org}",
            domain = config.domain,
            org = config.org,
        ))
        .header(reqwest::header::CONTENT_TYPE, "application/vnd.flux")
        .header(
            reqwest::header::AUTHORIZATION,
            format!("Token {}", config.token),
        )
        .body(q.to_string())
        .send()
        .await?
        .text()
        .await?;
    let mut reader = csv::Reader::from_reader(content.as_bytes());
    let row = reader
        .deserialize::<Row<T>>()
        .next()
        .context("no data")?
        .context("Invalid data")?;
    Ok(row)
}

#[tokio::test]
async fn get_data() {
    let client = reqwest::Client::new();
    let config = crate::config::Config::load("./config.toml").unwrap();
    let data = get(&client, &config.influx).await.unwrap();
    eprintln!("data = {data:#?}");
}
