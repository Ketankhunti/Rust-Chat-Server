// src/main.rs

use axum::{
    extract::{
        ws::{Message, Utf8Bytes, WebSocket},
        Path, State, WebSocketUpgrade,
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

// A type alias for the sender part of a WebSocket connection.
type ClientSender = SplitSink<WebSocket, Message>;

// A type alias for our shared state.
// The outer HashMap's key is the room name (a String).
// The value is another HashMap where the key is a client's UUID and the value is their sender.
type ChatState = Arc<Mutex<HashMap<String, HashMap<Uuid, ClientSender>>>>;

/// The main entry point for our application.
#[tokio::main]
async fn main() {
    // Create the shared state, an empty map of rooms.
    let state = ChatState::new(Mutex::new(HashMap::new()));

    // Define the application routes.
    // The route `/ws/{room}` captures a dynamic path segment.
    let app = Router::new()
        .route("/ws/{room}", get(websocket_handler))
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
/// It now uses `Path` to extract the room name from the URL.
async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<ChatState>,
    Path(room): Path<String>, // Extracts the `room` from `/ws/:room`
) -> impl IntoResponse {
    println!("New client connecting to room: {}", room);
    ws.on_upgrade(|socket| handle_socket(socket, state, room))
}

/// Handles an individual WebSocket connection, now aware of its room.
async fn handle_socket(socket: WebSocket, state: ChatState, room: String) {
    let client_id = Uuid::new_v4();
    println!("Client {} joined room '{}'", client_id, room);

    let (sender, receiver) = socket.split();

    // Add the new client to the state for the given room.
    {
        let mut rooms = state.lock().await;
        // `entry` gets or inserts a room, `or_default` creates a new HashMap if it doesn't exist.
        let room_clients = rooms.entry(room.clone()).or_default();
        room_clients.insert(client_id, sender);
    }

    // Spawn a task to handle messages from this client.
    let mut receive_task = tokio::spawn(read_from_client(receiver, client_id, state.clone(), room.clone()));

    // Wait for the client to disconnect.
    tokio::select! {
        _ = &mut receive_task => {
            // The receive task finished, indicating the client disconnected.
        }
    }

    // Client has disconnected, so we perform cleanup.
    println!("Client {} disconnected from room '{}'.", client_id, room);
    cleanup_client(&state, client_id, &room).await;
}

/// Reads messages from a client and broadcasts them to their room.
async fn read_from_client(
    mut receiver: SplitStream<WebSocket>,
    client_id: Uuid,
    state: ChatState,
    room: String,
) {
    while let Some(Ok(msg)) = receiver.next().await {
        if let Message::Text(text) = msg {
            println!("Received message from {} in room '{}': {}", client_id, room, text);
            // Broadcast the message to the client's room.
            broadcast_to_room(&text, client_id, &state, &room).await;
        }
    }
}

/// Broadcasts a message to all clients in a specific room, except the sender.
async fn broadcast_to_room(text: &str, sender_id: Uuid, state: &ChatState, room: &str) {
    let mut rooms = state.lock().await;

    if let Some(room_clients) = rooms.get_mut(room) {
        let broadcast_text = format!("[{}]: {}", sender_id, text);
        let message = Message::Text(Utf8Bytes::from(broadcast_text));

        // Iterate over all clients in the room.
        for (id, sender) in room_clients.iter_mut() {
            if *id != sender_id {
                if sender.send(message.clone()).await.is_err() {
                    // This client might have disconnected abruptly.
                    // The main `handle_socket` task will handle the final cleanup.
                    println!("Failed to send message to client {}", id);
                }
            }
        }
    }
}

/// Removes a client from the state and cleans up the room if it becomes empty.
async fn cleanup_client(state: &ChatState, client_id: Uuid, room: &str) {
    let mut rooms = state.lock().await;

    // Remove the client from the room.
    if let Some(room_clients) = rooms.get_mut(room) {
        room_clients.remove(&client_id);

        // If the room is now empty, remove the room itself.
        if room_clients.is_empty() {
            println!("Room '{}' is empty, removing it.", room);
            rooms.remove(room);
        }
    }
}
