// src/main.rs

mod database;
mod models;
mod state;
mod websocket;

use axum::{routing::get, Router};
use state::ChatState;
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;
use websocket::websocket_handler;

#[tokio::main]
async fn main() {
    // Set up the database connection pool.
    // .expect will crash the app if the DB can't be opened, which is reasonable on startup.
    let db_pool = database::setup_database()
        .await
        .expect("Failed to set up database");

    // The ChatState is a struct holding the map of rooms and the db_pool.
    let state = ChatState {
        rooms: Arc::new(Mutex::new(HashMap::new())),
        db_pool,
    };

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

