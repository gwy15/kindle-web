use std::borrow::Cow;

use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use anyhow::Result;

mod api;
mod config;

#[get("/kindle")]
async fn html() -> impl Responder {
    let cow: Cow<'static, [u8]> = tokio::fs::read("./static/index.html")
        .await
        .map(Into::into)
        .unwrap_or_else(|_| include_bytes!("../static/index.html").into());
    HttpResponse::Ok().body(cow)
}
#[get("/kindle/style.css")]
async fn css() -> impl Responder {
    let cow: Cow<'static, [u8]> = tokio::fs::read("./static/style.css")
        .await
        .map(Into::into)
        .unwrap_or_else(|_| include_bytes!("../static/index.html").into());
    HttpResponse::Ok().body(cow)
}

#[get("/kindle/data")]
async fn data(
    client: web::Data<reqwest::Client>,
    config: web::Data<config::Config>,
) -> impl Responder {
    #[derive(Debug, serde::Serialize)]
    struct Err {
        errmsg: String,
        #[serde(flatten)]
        data: api::Data<&'static str, &'static str>,
    }

    match api::get(&client, &config.influx).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(e) => HttpResponse::Ok().json(Err {
            errmsg: format!("{e:#?}"),
            data: api::Data {
                temp: "ERR",
                temp_time: chrono::Utc::now(),
                humid: "ERR",
                humid_time: chrono::Utc::now(),
            },
        }),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let config = config::Config::load("./config.toml")?;
    let config = web::Data::new(config);
    let client = web::Data::new(reqwest::Client::new());

    HttpServer::new(move || {
        App::new()
            .app_data(config.clone())
            .app_data(client.clone())
            .wrap(actix_web::middleware::Logger::default())
            .service(html)
            .service(css) //
            .service(data)
    })
    .bind(("::", 80))?
    .run()
    .await?;
    Ok(())
}
