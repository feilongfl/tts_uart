mod snr9816;

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use clap::Parser;
use serde::Deserialize;
use std::sync::Mutex;

#[allow(unused_imports)]
use log::{debug, error, info};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Config {
    #[clap(long)]
    serial_port: String,

    #[clap(long, default_value_t = 115200)]
    baud_rate: u32,

    #[clap(long, default_value_t = String::from("127.0.0.1"))]
    server_addr: String,

    #[clap(long, default_value_t = 8080)]
    server_port: u16,
}

#[derive(Deserialize, Debug, Clone)]
struct TtsRequest {
    text: String,
}

async fn index() -> impl Responder {
    HttpResponse::Ok().body("OK")
}

async fn post_v1_tts_snr9816(
    dev: web::Data<Mutex<snr9816::SNR9816>>,
    info: web::Form<TtsRequest>,
) -> impl Responder {
    let mut dev = dev.lock().expect("Failed to lock serial");
    debug!("/v1/tts/snr9816[POST]: {}", info.text);
    dev.volume(2).await;
    dev.message(5).await;
    dev.tts(info.text.clone()).await;

    HttpResponse::Ok().body("OK")
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let config = Config::parse();
    env_logger::init();

    info!("open uart {}:{}", config.serial_port, config.baud_rate);
    let tts_dev = web::Data::new(Mutex::new(snr9816::SNR9816::new(
        config.serial_port,
        config.baud_rate,
    )));

    info!("listen {}:{}", config.server_addr, config.server_port);
    HttpServer::new(move || {
        App::new()
            .app_data(tts_dev.clone())
            .route("/", web::get().to(index))
            .route("/v1/tts/snr9816", web::get().to(index))
            .route("/v1/tts/snr9816", web::post().to(post_v1_tts_snr9816))
    })
    .bind((config.server_addr, config.server_port))?
    .run()
    .await
}
