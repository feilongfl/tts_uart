use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about, long_about = None)]
pub struct Config {
    #[clap(long)]
    pub serial_port: String,

    #[clap(long, default_value_t = 115200)]
    pub baud_rate: u32,

    #[clap(long, default_value_t = String::from("127.0.0.1"))]
    pub server_addr: String,

    #[clap(long, default_value_t = 8080)]
    pub server_port: u16,

    #[clap(short, long, default_value_t=2)]
    pub volume: u8,

    #[clap(short, long, default_value_t=2)]
    pub speed: u8,

    #[clap(short, long, default_value_t=2)]
    pub tone: u8,

    #[clap(short, long, default_value_t=String::from("message_5"))]
    pub notify: String,
}
