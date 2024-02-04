use std::io::Write;

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use clap::Parser;
use encoding_rs::GBK;
use serde::Deserialize;
use std::sync::Mutex;
use tokio_serial::{SerialPortBuilderExt, SerialStream};
use log::{debug, error, log_enabled, info, Level};

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

#[derive(Deserialize, Debug)]
struct TtsRequest {
    text: String,
}

async fn index() -> impl Responder {
    HttpResponse::Ok().body("OK")
}

struct GbkEncodedString {
    buffer: Vec<u8>,
}

impl GbkEncodedString {
    fn new(input: &str) -> Self {
        let (encoded, _, _) = GBK.encode(input); // 使用encoding_rs进行编码

        GbkEncodedString {
            buffer: encoded.to_vec(),
        }
    }

    fn to_snr9816(&self) -> Vec<u8> {
        let length = self.buffer.len();
        if length > u16::MAX as usize {
            panic!("String too long to encode with u16 length prefix");
        }

        let mut result = Vec::with_capacity(length + 5);
        result.push(0xfd); // head

        result.push((length >> 8) as u8); // len[15:8]
        result.push(length as u8); // len[7:0]

        result.push(0x01); // command
        result.push(0x01); // codec

        result.extend_from_slice(&self.buffer);
        result
    }
}

async fn post_tts(
    serial: web::Data<Mutex<SerialStream>>,
    info: web::Form<TtsRequest>,
) -> impl Responder {
    let str = GbkEncodedString::new(info.text.as_str());
    let mut serial = serial.lock().expect("Failed to lock serial");
    debug!("/v1/tts/snr9816[POST]: {}", info.text);
    serial.write_all(&str.to_snr9816()).expect("Failed to write to serial port");

    HttpResponse::Ok().body("OK")
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let config = Config::parse();
    env_logger::init();

    info!("open uart {}:{}", config.serial_port, config.baud_rate);
    let port = tokio_serial::new(config.serial_port, config.baud_rate).open_native_async()?;
    let serial = web::Data::new(Mutex::new(port));

    info!("listen {}:{}", config.server_addr, config.server_port);
    HttpServer::new(move || {
        App::new()
            .app_data(serial.clone())
            .route("/", web::get().to(index))
            .route("/v1/tts/snr9816", web::get().to(index))
            .route("/v1/tts/snr9816", web::post().to(post_tts))
    })
    .bind((config.server_addr, config.server_port))?
    .run()
    .await
}
