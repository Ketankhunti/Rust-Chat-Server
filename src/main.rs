// src/main.rs

// Declare the modules we're using
mod models;
mod state;
mod websocket;

use axum::{routing::get, Router};
use state::ChatState;
use std::{collections::HashMap, net::SocketAddr};
use tokio::sync::Mutex;
use websocket::websocket_handler;

#[tokio::main]
async fn main() {
    // Initialize the shared state (an empty map of rooms)
    let state = ChatState::new(Mutex::new(HashMap::new()));

    // Define the application routes
    let app = Router::new()
        .route("/ws/{room}", get(websocket_handler))
        .with_state(state);

    // Define the server address
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("WebSocket server listening on ws://{}...", addr);

    // Create a TCP listener and start the server
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind address");

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
