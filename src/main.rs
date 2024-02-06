mod read_socket;
mod server;
mod server_join_handler;
mod server_publisher_listener;

use clap::Parser;
use log::info;
use simplelog::*;
use std::net::{IpAddr, SocketAddr};
use tokio::net::TcpListener;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(name = "Simplified Kafka")]
#[clap(author = "Alexey Datskovskiy")]
#[clap(version = "0.2.0")]
#[clap(
    about = "Kafka server",
    long_about = r#"Simplified Kafka server
Supports several topics, publishers and subscribers

To join the topic as a subscriber, send {"method": "subscribe", "topic": "[topic_name]"}
To join the topic as a publisher, send {"method": "publish", "topic": "[topic_name]"}
To send a message as a publisher, send {"message": "[your_message_to_the_world]"}
Messages will be sent to subscribers in json: {"message": "[someones_message_to_the_world]"}
Any errors will be send in json: {"error": "[error_description]"}

Use \\n as a separator"#
)]
struct Args {
    /// Address for the server
    #[arg(long, default_value = "127.0.0.1")]
    address: IpAddr,

    /// Port for the server
    #[arg(long, default_value = "8080")]
    port: u16,
}

#[tokio::main]
async fn main() {
    TermLogger::init(
        LevelFilter::Info,
        Config::default(),
        TerminalMode::Stderr,
        ColorChoice::Auto,
    )
    .unwrap();

    let args = Args::parse();

    let server = TcpListener::bind(SocketAddr::new(args.address, args.port)).await;
    match server {
        Err(e) => match e.kind() {
            std::io::ErrorKind::AddrInUse => info!(
                "Not able to start server on {}:{} -- port is busy",
                args.address, args.port
            ),
            _ => info!(
                "Not able to start server on {}:{} - {:?}",
                args.address, args.port, e
            ),
        },
        Ok(server) => {
            info!(
                "Start kafka server on address {}:{}",
                args.address, args.port
            );
            server::process_messages(server).await;
        }
    }
}
