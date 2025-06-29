// src/websocket.rs

use crate::{
    models::ServerMessage,
    state::{ChatState, Client},
};
use axum::{
    extract::{
        ws::{Message, WebSocket},
        Path, State, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use futures_util::{
    sink::SinkExt,
    stream::{SplitStream, StreamExt},
};
use std::collections::HashMap;
use tokio::sync::MutexGuard;
use uuid::Uuid;

/// The main handler for WebSocket connections.
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<ChatState>,
    Path(room): Path<String>,
) -> impl IntoResponse {
    println!("New client connecting to room: {}", room);
    ws.on_upgrade(|socket| handle_socket(socket, state, room))
}

/// Manages the lifecycle of a client. A client is anonymous until they set a username.
async fn handle_socket(socket: WebSocket, state: ChatState, room: String) {
    let client_id = Uuid::new_v4();
    let (sender, receiver) = socket.split();

    // Add the client to the state as "anonymous" immediately.
    {
        let mut rooms = state.lock().await;
        let client = Client {
            username: "anonymous".to_string(),
            sender,
        };
        rooms.entry(room.clone()).or_default().insert(client_id, client);
        println!("Client {} connected to room '{}' as anonymous.", client_id, room);
    }

    // Spawn the task to handle all messages from this client.
    let mut receive_task =
        tokio::spawn(read_from_client(receiver, client_id, state.clone(), room.clone()));

    // Wait for the client to disconnect.
    tokio::select! {
        _ = &mut receive_task => {}
    }

    // Client has disconnected, perform cleanup.
    cleanup_client(&state, client_id, &room).await;
}

/// Reads messages from a client and processes them as commands or chat messages.
async fn read_from_client(
    mut receiver: SplitStream<WebSocket>,
    client_id: Uuid,
    state: ChatState,
    room: String,
) {
    while let Some(Ok(Message::Text(text))) = receiver.next().await {
        let text = text.trim(); // Trim whitespace from the message

        if text.starts_with("/user ") {
            // Get the text that comes after "/user "
            let username = &text[6..].trim(); // Use slicing for robustness
            if !username.is_empty() {
                handle_set_username(username.to_string(), client_id, &state, &room).await;
            }
        } else {
            // Treat as a regular chat message.
            handle_chat_message(text.to_string(), client_id, &state, &room).await;
        }
    }
}


/// Handles setting or updating a client's username.
async fn handle_set_username(username: String, client_id: Uuid, state: &ChatState, room: &str) {
    let mut rooms = state.lock().await;
    let mut old_username = "anonymous".to_string();

    if let Some(client) = rooms.get_mut(room).and_then(|r| r.get_mut(&client_id)) {
        old_username = client.username.clone();
        client.username = username.clone();
    }

    println!("Client {} ({}) is now known as '{}' in room '{}'", client_id, old_username, username, room);

    // Announce the user has joined/changed their name.
    let join_msg = ServerMessage::UserJoined { username };
    // Exclude the sender from receiving their own join message
    broadcast_message(join_msg, &mut rooms, room, Some(client_id)).await;
}

/// Handles a regular chat message from a client.
async fn handle_chat_message(content: String, client_id: Uuid, state: &ChatState, room: &str) {
    let mut rooms = state.lock().await;
    
    if let Some(client) = rooms.get(room).and_then(|r| r.get(&client_id)) {
        let username = client.username.clone();
        
        if username == "anonymous" {
            // Prevent anonymous users from sending messages.
            if let Some(client) = rooms.get_mut(room).and_then(|r| r.get_mut(&client_id)) {
                let _ = client.sender.send(Message::Text("Please set a username with `/user <name>` before sending messages.".to_string().into())).await;
            }
            return;
        }
        
        // Check if the message is empty or only whitespace
        if content.trim().is_empty() {
            return; // Don't broadcast empty messages
        }
        
        println!("Message from {}({}): {}", username, client_id, &content);
        let new_msg = ServerMessage::NewMessage { username, content };
        // Exclude the sender from receiving their own message
        broadcast_message(new_msg, &mut rooms, room, Some(client_id)).await;
    }
}

/// Serializes and broadcasts a `ServerMessage` to all clients in a room.
async fn broadcast_message(
    message: ServerMessage,
    rooms: &mut MutexGuard<'_, HashMap<String, HashMap<Uuid, Client>>>,
    room: &str,
    exclude_client_id: Option<Uuid>,
) {
    if let Some(room_clients) = rooms.get_mut(room) {
        let parsed_message = parse_message_for_display(&message);

        for (id, client) in room_clients.iter_mut() {
            // Skip sending to the excluded client (if any)
            if exclude_client_id.map_or(false, |exclude_id| *id == exclude_id) {
                continue;
            }

            // Send parsed message for easier terminal testing
            if client.sender.send(Message::Text(parsed_message.clone().into())).await.is_err() {
                println!("Failed to send parsed message to client {}", id);
            }
        }
    }
}

/// Converts a ServerMessage to a human-readable format for testing.
fn parse_message_for_display(message: &ServerMessage) -> String {
    match message {
        ServerMessage::NewMessage { username, content } => {
            format!("[{}] {}", username, content)
        }
        ServerMessage::UserJoined { username } => {
            format!("--> {} joined the room", username)
        }
        ServerMessage::UserLeft { username } => {
            format!("<-- {} left the room", username)
        }
    }
}

/// Removes a client from the state and announces their departure.
async fn cleanup_client(state: &ChatState, client_id: Uuid, room: &str) {
    let mut username = "anonymous".to_string();
    let mut should_broadcast = false;

    // First, remove the client and get their username
    {
        let mut rooms = state.lock().await;
        if let Some(room_clients) = rooms.get_mut(room) {
            // Remove the client and get their username.
            if let Some(client) = room_clients.remove(&client_id) {
                username = client.username;
                should_broadcast = username != "anonymous";
            }

            // If the room is now empty, remove it from the state.
            if room_clients.is_empty() {
                println!("Room '{}' is empty, removing it.", room);
                rooms.remove(room);
            }
        }
    } // First lock is released here

    // Now broadcast departure message with a fresh lock
    if should_broadcast {
        println!("Broadcasting leave message for {} from room '{}'", username, room);
        let left_msg = ServerMessage::UserLeft { username: username.clone() };
        let mut rooms_for_broadcast = state.lock().await;
        broadcast_message(left_msg, &mut rooms_for_broadcast, room, None).await;
    }

    println!(
        "Client {} ({}) disconnected from room '{}'.",
        client_id, username, room
    );
}
