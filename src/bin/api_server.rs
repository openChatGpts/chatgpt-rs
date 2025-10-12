use chatgpt_rs::api::server;
use chatgpt_rs::{log_error, log_info, log_success};
use std::env;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let host = env::var("API_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("API_PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(6969);

    log_info!("Starting API server on {}:{}", host, port);

    if let Err(err) = server::run(&host, port).await {
        log_error!("API server failed: {}", err);
        std::process::exit(1);
    }
    log_success!("API server stopped");
}
