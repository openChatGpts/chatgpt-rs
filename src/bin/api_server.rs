use chatgpt_rs::api::server;
use chatgpt_rs::{log_error, log_info, log_success};
use std::env;

fn print_usage() {
    println!("Usage: api_server [OPTIONS]");
    println!();
    println!("Options:");
    println!("  --host <HOST>      Server host (default: 0.0.0.0)");
    println!("  --port <PORT>      Server port (default: 6969)");
    println!("  --proxy <PROXY>    Default proxy URL (default: http://127.0.0.1:1082)");
    println!("  --help             Show this help message");
    println!();
    println!("Examples:");
    println!("  api_server");
    println!("  api_server --port 8080");
    println!("  api_server --proxy http://proxy.example.com:8080");
    println!("  api_server --host 127.0.0.1 --port 8080 --proxy http://localhost:7890");
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let args: Vec<String> = env::args().collect();
    
    // Parse command line arguments
    let mut host = "0.0.0.0".to_string();
    let mut port = 6969u16;
    let mut default_proxy = Some("http://127.0.0.1:1082".to_string());
    
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => {
                print_usage();
                return;
            }
            "--host" => {
                if i + 1 < args.len() {
                    host = args[i + 1].clone();
                    i += 2;
                } else {
                    log_error!("--host requires a value");
                    std::process::exit(1);
                }
            }
            "--port" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<u16>() {
                        Ok(p) => port = p,
                        Err(_) => {
                            log_error!("Invalid port number: {}", args[i + 1]);
                            std::process::exit(1);
                        }
                    }
                    i += 2;
                } else {
                    log_error!("--port requires a value");
                    std::process::exit(1);
                }
            }
            "--proxy" => {
                if i + 1 < args.len() {
                    default_proxy = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    log_error!("--proxy requires a value");
                    std::process::exit(1);
                }
            }
            "--no-proxy" => {
                default_proxy = None;
                i += 1;
            }
            _ => {
                log_error!("Unknown option: {}", args[i]);
                println!();
                print_usage();
                std::process::exit(1);
            }
        }
    }

    // Environment variables can override command line args
    if let Ok(env_host) = env::var("API_HOST") {
        host = env_host;
    }
    
    if let Ok(env_port) = env::var("API_PORT") {
        if let Ok(p) = env_port.parse::<u16>() {
            port = p;
        }
    }
    
    if let Ok(env_proxy) = env::var("DEFAULT_PROXY") {
        default_proxy = Some(env_proxy);
    }

    log_info!("Starting ChatGPT-RS API Server");
    log_info!("================================");
    log_info!("Host: {}", host);
    log_info!("Port: {}", port);
    
    if let Some(ref proxy) = default_proxy {
        log_info!("Default Proxy: {}", proxy);
    } else {
        log_info!("Default Proxy: None");
    }
    
    log_info!("================================");
    log_info!("Endpoints:");
    log_info!("  POST /v1/chat/completions - OpenAI-compatible chat completions");
    log_info!("================================");

    if let Err(err) = server::run(&host, port, default_proxy).await {
        log_error!("API server failed: {}", err);
        std::process::exit(1);
    }
    log_success!("API server stopped");
}
