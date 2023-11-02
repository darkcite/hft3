use std::collections::{HashMap, HashSet};
use std::f64::consts::E;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, WebSocketStream};
use tokio::net::TcpStream;
use url::Url;
use futures_util::stream::StreamExt;

// Helper function to extract currency pair from a symbol like "BTCUSDT"
fn extract_currency_pair(symbol: &str) -> (String, String) {
    let base = &symbol[0..3];
    let quote = &symbol[3..];
    (base.to_string(), quote.to_string())
}

// TickerData struct corresponding to Binance ticker format
#[derive(serde::Deserialize, Debug)]
struct TickerData {
    s: String, // Symbol
    c: String, // Last price as a string to handle precision
    // You can add more fields if needed
}

struct Edge {
    start: String,
    end: String,
    rate: f64,
}

struct Graph {
    edges: Vec<Edge>,
    vertices: HashSet<String>,
}

impl Graph {
    fn new() -> Self {
        Graph {
            edges: Vec::new(),
            vertices: HashSet::new(),
        }
    }

    fn add_edge(&mut self, start: String, end: String, rate: f64) {
        self.vertices.insert(start.clone());
        self.vertices.insert(end.clone());
        self.edges.push(Edge { start, end, rate });
    }

    fn update_edge(&mut self, start: &str, end: &str, rate: f64) {
        if let Some(edge) = self.edges.iter_mut().find(|e| &e.start == start && &e.end == end) {
            edge.rate = rate;
        }
    }

    fn find_arbitrage(&self) -> Option<Vec<String>> {
        let mut distances = HashMap::new();
        let mut predecessors = HashMap::new();
    
        // Initialize distances to infinity, and set the distance to a starting node to 0
        let start_vertex = self.vertices.iter().next()?.clone();
        for vertex in &self.vertices {
            distances.insert(vertex.clone(), f64::INFINITY);
            predecessors.insert(vertex.clone(), None);
        }
        distances.insert(start_vertex.clone(), 0.0);
    
        // Relax edges repeatedly
        for _ in 1..self.vertices.len() {
            for edge in &self.edges {
                // Compute the new distance considering the logarithm of the edge rate
                let weight = -edge.rate.log(E);
                let new_dist = distances[&edge.start] + weight;
                
                // Check for overflow/underflow or any other arithmetic issues
                if new_dist.is_finite() && new_dist < distances[&edge.end] {
                    distances.insert(edge.end.clone(), new_dist);
                    predecessors.insert(edge.end.clone(), Some(edge.start.clone()));
                }
            }
        }
    
        // Check for negative-weight cycles
        for edge in &self.edges {
            let weight = -edge.rate.log(E);
            let new_dist = distances[&edge.start] + weight;
            
            if new_dist.is_finite() && new_dist < distances[&edge.end] {
                // We found a cycle, now reconstruct the path
                let mut cycle = vec![edge.end.clone()];
                let mut last = edge.end.clone();
                while let Some(pred) = predecessors[&last].clone() {
                    if cycle.contains(&pred) {
                        cycle.push(pred);
                        cycle.reverse();
                        return Some(cycle); // Return the cycle representing the arbitrage opportunity
                    }
                    cycle.push(pred.clone());
                    last = pred;
                }
                break;
            }
        }
    
        // If we reach this point, no arbitrage opportunity was found
        None
    }    
}

// Function to process ticker data and update the graph
async fn process_ticker_data(graph: &mut Graph, ticker_data: Vec<TickerData>) {
    for data in ticker_data {
        let (start, end) = extract_currency_pair(&data.s); // Use 's' for symbol
        let price: f64 = match data.c.parse() { // Parse the last price from string to f64
            Ok(p) => p,
            Err(_) => {
                eprintln!("Error parsing price for symbol {}", &data.s);
                continue; // Skip this entry if the price can't be parsed
            }
        };
        graph.update_edge(&start, &end, price);
    }

    // Here you could check for arbitrage opportunities
    if let Some(arbitrage_path) = graph.find_arbitrage() {
        println!("Arbitrage opportunity found: {:?}", arbitrage_path);
        // Handle the arbitrage opportunity
    }
}

// Function to listen to the WebSocket stream and update the graph
async fn listen_to_stream_and_update_graph(graph: &mut Graph, ws_stream: WebSocketStream<tokio_tungstenite::MaybeTlsStream<TcpStream>>) {
    let (_, mut read) = ws_stream.split();

    // Read messages from the stream
    while let Some(message) = read.next().await {
        match message {
            Ok(msg) => {
                if msg.is_text() || msg.is_binary() {
                    // println!("Received a message: {:?}", msg);
                    let ticker_data: Vec<TickerData> = match serde_json::from_str(&msg.to_text().unwrap()) {
                        Ok(data) => data,
                        Err(e) => {
                            eprintln!("Error parsing ticker data: {:?}", e);
                            continue; // Skip this message and continue with the next
                        }
                    };
                    process_ticker_data(graph, ticker_data).await;
                }
            }
            Err(e) => {
                eprintln!("Error receiving message: {:?}", e);
                break;
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let mut graph = Graph::new();

    // Connect to the WebSocket stream
    let binance_ws_url = "wss://stream.binance.com:9443/ws/!ticker@arr";
    let url = Url::parse(binance_ws_url).expect("Failed to parse URL");
    let (ws_stream, _) = connect_async(url)
        .await
        .expect("Failed to connect to Binance WebSocket");
    println!("Connected to the Binance WebSocket server");

    // Start listening to the stream and updating the graph
    listen_to_stream_and_update_graph(&mut graph, ws_stream).await;
}
