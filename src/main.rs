// src/main.rs

use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use futures_util::{sink::SinkExt, stream::StreamExt}; // For stream/sink methods
use std::net::SocketAddr;
use tokio::net::TcpListener;

/// The main entry point for our application.
#[tokio::main]
async fn main() {
    // Define the application routes.
    // For this phase, we only have one route: a GET request to "/ws"
    // which will be handled by our `websocket_handler` function.
    let app = Router::new().route("/ws", get(websocket_handler));

    // Define the address to listen on.
    // "0.0.0.0" means it will listen on all available network interfaces.
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("WebSocket server listening on ws://{}...", addr);

    // Create a TCP listener and bind it to the address.
    let listener = TcpListener::bind(&addr).await.expect("Failed to bind address");

    // Start the server, passing it the defined routes.
    // This will block indefinitely, handling incoming connections.
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

/// The handler for the WebSocket route.
///
/// This function is called when a client tries to connect to "/ws".
/// It uses the `WebSocketUpgrade` extractor to upgrade the HTTP connection
/// into a persistent WebSocket connection.
async fn websocket_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    // The `on_upgrade` method takes a callback that will be executed
    // once the WebSocket handshake is successful.
    // The `handle_socket` function will be our callback.
    ws.on_upgrade(handle_socket)
}

/// Handles an individual WebSocket connection.
///
/// This function is the core of our echo server. It takes ownership of the
/// WebSocket and enters a loop to process messages.
async fn handle_socket(mut socket: WebSocket) {
    println!("New client connected!");

    // The `split` method is useful for handling reads and writes concurrently
    // if we were to use `tokio::spawn`, but for a simple echo server,
    // we can just use the mutable `socket` directly.

    // This loop will run as long as the client is connected and sending messages.
    // `socket.next().await` waits for the next message from the client.
    // It returns `Some(Ok(message))` if a message is received,
    // `Some(Err(e))` if there's an error, and `None` if the connection is closed.
    while let Some(Ok(msg)) = socket.next().await {
        match msg {
            // We only process Text messages. Binary, Ping, Pong, and Close
            // messages are ignored for this simple example.
            Message::Text(text) => {
                println!("Received message: {}", text);

                // Echo the message back to the client.
                // We send the message back using `socket.send()`.
                // `Message::Text` creates a text frame.
                if socket.send(Message::Text(text)).await.is_err() {
                    // If `send` returns an error, it means the client has
                    // disconnected, so we can break the loop.
                    println!("Client disconnected unexpectedly.");
                    break;
                }
            }
            Message::Close(_) => {
                // The client sent a Close frame.
                println!("Client requested to close connection.");
                break; // Exit the loop to close the connection.
            }
            // Ignore other message types for simplicity
            _ => {}
        }
    }

    // This part of the code is reached when the loop breaks, which means
    // the client has disconnected.
    println!("Client connection closed.");
}
