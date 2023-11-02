use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::connect_async;
use url::Url;

#[tokio::main]
async fn main() {
    // Subscribe to the all market tickers stream
    let binance_ws_url = "wss://stream.binance.com:9443/ws/!ticker@arr";
    
    let url = Url::parse(binance_ws_url).expect("Failed to parse URL");

    let (ws_stream, _) = connect_async(url)
        .await
        .expect("Failed to connect to Binance WebSocket");
    
    println!("Connected to the Binance WebSocket server");

    let (_, mut read) = ws_stream.split();

    // Read messages from the stream
    while let Some(message) = read.next().await {
        match message {
            Ok(msg) => {
                if msg.is_text() || msg.is_binary() {
                    println!("Received a message: {:?}", msg);
                }
            }
            Err(e) => {
                eprintln!("Error receiving message: {:?}", e);
                break;
            }
        }
    }
}
