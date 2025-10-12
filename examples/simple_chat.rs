use chatgpt_rs::client::ChatGptClient;
use chatgpt_rs::{log_error, log_info, log_success};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Example proxy (replace with your own)
    let proxy = "http://127.0.0.1:1082";

    log_info!("Creating ChatGPT client with proxy: {}", proxy);

    // Create client
    let mut client = ChatGptClient::new(Some(proxy)).await?;

    log_success!("Client created successfully!");

    // Ask a question
    let question = "你是怎么看到openai这个公司的";
    log_info!("Asking question: {}", question);

    match client.ask_question(question).await {
        Ok(response) => {
            log_success!("Got response:");
            println!("\n{}\n", response);
        }
        Err(e) => {
            log_error!("Error: {}", e);
        }
    }

    Ok(())
}
