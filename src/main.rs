// src/main.rs

use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use futures_util::{
    sink::SinkExt,
    stream::{SplitSink, SplitStream, StreamExt},
};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;
use uuid::Uuid;

// We use a type alias for our shared state to make the code cleaner.
// It's a thread-safe, shared map from a client's unique ID to their WebSocket sender.
type ChatState = Arc<Mutex<HashMap<Uuid, SplitSink<WebSocket, Message>>>>;

/// The main entry point for our application.
#[tokio::main]
async fn main() {
    // Create the shared state. We wrap our HashMap in a Mutex and an Arc.
    let state = ChatState::new(Mutex::new(HashMap::new()));

    // Define the application routes.
    // We pass our shared state to the handler using `with_state`.
    let app = Router::new()
        .route("/ws", get(websocket_handler))
        .with_state(state);

    // Define the address to listen on.
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("WebSocket server listening on ws://{}...", addr);

    // Start the server.
    let listener = tokio::net::TcpListener::bind(&addr).await.expect("Failed to bind address");
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

/// The handler for the WebSocket route.
/// It now accepts the shared `ChatState` as an argument.
async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<ChatState>, // axum's State extractor
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

/// Handles an individual WebSocket connection.
///
/// This function now contains the core logic for a client's lifecycle.
async fn handle_socket(socket: WebSocket, state: ChatState) {
    // Generate a unique ID for this new client.
    let client_id = Uuid::new_v4();
    println!("New client connected: {}", client_id);

    // Split the WebSocket into a sender and a receiver.
    // The sender can be cloned and shared, while the receiver is unique to this task.
    let (sender, receiver) = socket.split();

    // Store the sender part of the socket in our shared state.
    // We lock the mutex to ensure exclusive access.
    {
        let mut clients = state.lock().await;
        clients.insert(client_id, sender);
    } // The lock is released here as `clients` goes out of scope.

    // This task will handle messages coming *from* the client.
    let mut receive_task = tokio::spawn(read_from_client(receiver, client_id, state.clone()));

    // Wait for the receive task to complete. This happens when the client disconnects.
    tokio::select! {
        _ = &mut receive_task => {
            // The receive task finished, which means the client disconnected.
        }
    }

    // If we get here, the client has disconnected.
    // We need to remove them from the shared state.
    println!("Client {} disconnected.", client_id);
    let mut clients = state.lock().await;
    clients.remove(&client_id);
}

/// Reads messages from a client and broadcasts them to others.
async fn read_from_client(
    mut receiver: SplitStream<WebSocket>,
    client_id: Uuid,
    state: ChatState,
) {
    while let Some(Ok(msg)) = receiver.next().await {
        if let Message::Text(text) = msg {
            println!("Received message from {}: {}", client_id, text);
            // Broadcast the message to all other connected clients.
            broadcast_message(text.to_string(), client_id, &state).await;
        }
    }
}

/// Broadcasts a message to all connected clients except the sender.
async fn broadcast_message(text: String, sender_id: Uuid, state: &ChatState) {
    let mut clients = state.lock().await;

    // We create a message to be sent. We can add more info here later.
    let broadcast_text = format!("[{}]: {}", sender_id, text);
    let message = Message::Text(broadcast_text.into());

    // Iterate over all connected clients.
    for (id, sender) in clients.iter_mut() {
        // Don't send the message back to the original sender.
        if *id != sender_id {
            // Send the message. If it fails, the client might have disconnected
            // abruptly. We can log this but won't handle removal here, as the
            // main `handle_socket` task will take care of cleanup.
            if sender.send(message.clone()).await.is_err() {
                println!("Failed to send message to client {}", id);
            }
        }
    }
}
