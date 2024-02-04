mod args;
mod snr9816;

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use clap::Parser;
use serde::Deserialize;
use std::sync::Mutex;

#[allow(unused_imports)]
use log::{debug, error, info};

#[derive(Deserialize, Debug, Clone)]
struct TtsRequest {
    text: String,
}

async fn index() -> impl Responder {
    HttpResponse::Ok().body("OK")
}

async fn post_v1_tts_snr9816(
    config: web::Data<args::Config>,
    dev: web::Data<Mutex<snr9816::SNR9816>>,
    info: web::Form<TtsRequest>,
) -> impl Responder {
    let mut dev = dev.lock().expect("Failed to lock serial");
    debug!("/v1/tts/snr9816[POST]: {}", info.text);
    info!("config: {:?}", config);
    dev.volume(config.volume.clone()).await;
    dev.speed(config.speed.clone()).await;
    dev.tone(config.tone.clone()).await;
    dev.notify(config.notify.clone()).await;
    dev.tts(info.text.clone()).await;

    HttpResponse::Ok().body("OK")
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let config = args::Config::parse();
    let _ =
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    info!("open uart {}:{}", config.serial_port, config.baud_rate);
    let tts_dev = web::Data::new(Mutex::new(snr9816::SNR9816::new(
        config.serial_port.clone(),
        config.baud_rate,
    )));

    let config_webdata = web::Data::new(config.clone());

    info!("listen {}:{}", config.server_addr, config.server_port);
    HttpServer::new(move || {
        App::new()
            .app_data(config_webdata.clone())
            .app_data(tts_dev.clone())
            .route("/", web::get().to(index))
            .route("/v1/tts/snr9816", web::get().to(index))
            .route("/v1/tts/snr9816", web::post().to(post_v1_tts_snr9816))
    })
    .bind((config.server_addr, config.server_port))?
    .workers(1)
    .run()
    .await
}
