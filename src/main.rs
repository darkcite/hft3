use futures_util::{stream::StreamExt, sink::SinkExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use url::Url;

#[tokio::main]
async fn main() {
    // The WebSocket endpoint for Binance streams
    let binance_ws_url = "wss://stream.binance.com:9443/ws/btcusdt@trade";

    // Parse the URL for the WebSocket connection
    let url = Url::parse(binance_ws_url).expect("Failed to parse URL");

    // Connect to the WebSocket server asynchronously
    let (ws_stream, _) = connect_async(url)
        .await
        .expect("Failed to connect to Binance WebSocket");

    println!("Connected to the Binance WebSocket API");

    let (write, read) = ws_stream.split();

    // Listen for messages
    let read_messages = read.for_each(|message| async {
        match message {
            Ok(msg) => {
                if msg.is_text() {
                    println!("Received a message: {}", msg.to_text().unwrap());
                }
            }
            Err(e) => {
                eprintln!("Error during the WebSocket connection: {:?}", e);
                // If you want to stop listening on errors, uncomment the line below
                // return futures_util::future::ready(());
            }
        }
    });

    // You can also send messages using the write part of the split, but for now we are just reading.
    // let _ = write.send(Message::Text("Some message".to_string())).await;

    read_messages.await;

    println!("WebSocket connection closed");
}
