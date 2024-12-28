use std::io::Write;
use std::sync::Arc;
use std::time::Instant;
use colored::Colorize;
use log::{Level, LevelFilter};
use tokio::net::{TcpListener};
use necko_core::client::Client;
use chrono::Local;
use necko_core::server::Server;
use necko_core::server::status::MINECRAFT_VERSION;

const TCP_ADDRESS: &str = "127.0.0.1:25565";

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let time = Instant::now();
    init_logger(LevelFilter::Debug);
    log::info!("Starting necko-server v{} on Minecraft {MINECRAFT_VERSION}", env!("CARGO_PKG_VERSION"));

    let listener = TcpListener::bind(TCP_ADDRESS).await
        .expect("Could not bind TCP listener");
    log::info!("Bound TCP socket on {}", listener.local_addr().expect("Could not get local address"));
    
    let server = Arc::new(Server::new());
    
    log::info!("Server started in {}s", time.elapsed().as_secs_f32());
    loop {
        let (stream, addr) = listener.accept().await?;
        log::debug!("Accepted connection from {}", addr);

        stream.set_nodelay(true).unwrap_or_else(
            |e| log::warn!("Failed to set TCP_NODELAY on socket: {}", e));

        let client = Arc::new(Client::new(stream, addr));

        let server = server.clone();
        tokio::spawn(async move {
            log::debug!("Starting polling for {:?}", client.address.read().await);
            while !client.closed.load(std::sync::atomic::Ordering::Relaxed) {
                let done = client.poll().await;
                if done {
                    client.process_packets(&server).await;
                }
            }
        });
    }
}

fn init_logger(level: LevelFilter) {
    env_logger::Builder::from_default_env()
        .format(|buf, record| {
            let message = format!(
                "[{} {}]: {}",
                Local::now().format("%H:%M:%S"),
                record.level(),
                record.args()
            );

            let colored_message = match record.level() {
                Level::Error => message.bright_red(),
                Level::Warn => message.yellow(),
                Level::Info => message.white(),
                Level::Debug => message.white().dimmed(),
                Level::Trace => message.bright_black(),
            };

            writeln!(buf, "{}", colored_message)
        })
        .filter_level(level)
        .init();
}

