// src/websocket.rs

use crate::{
    models::ServerMessage,
    state::{ChatState, Client, Room},
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
use std::collections::{HashMap, VecDeque};
use tokio::sync::MutexGuard;
use uuid::Uuid;

const MAX_HISTORY_SIZE: usize = 10;

/// The main handler for WebSocket connections.
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<ChatState>,
    Path(room_name): Path<String>,
) -> impl IntoResponse {
    println!("New client connecting to room: {}", room_name);
    ws.on_upgrade(|socket| handle_socket(socket, state, room_name))
}

/// Manages the lifecycle of a client. A client is anonymous until they set a username.
async fn handle_socket(socket: WebSocket, state: ChatState, room_name: String) {
    let client_id = Uuid::new_v4();
    let (sender, receiver) = socket.split();

    // Add the client to the state as "anonymous" immediately.
    {
        let mut rooms = state.lock().await;
        let room = rooms.entry(room_name.clone()).or_default();
        let client = Client {
            username: "anonymous".to_string(),
            sender,
        };
        room.clients.insert(client_id, client);
        println!("Client {} connected to room '{}' as anonymous.", client_id, room_name);
    }

    // Spawn the task to handle all messages from this client.
    let mut receive_task =
        tokio::spawn(read_from_client(receiver, client_id, state.clone(), room_name.clone()));

    // Wait for the client to disconnect.
    tokio::select! {
        _ = &mut receive_task => {}
    }

    // Client has disconnected, perform cleanup.
    cleanup_client(&state, client_id, &room_name).await;
}

/// Reads messages from a client and processes them as commands or chat messages.
async fn read_from_client(
    mut receiver: SplitStream<WebSocket>,
    client_id: Uuid,
    state: ChatState,
    room_name: String,
) {
    while let Some(Ok(Message::Text(text))) = receiver.next().await {
        let text = text.trim();

        if text.starts_with("/user ") {
            if let Some(username) = text.strip_prefix("/user ").and_then(|s| {
                let trimmed = s.trim();
                if !trimmed.is_empty() { Some(trimmed) } else { None }
            }) {
                handle_set_username(username.to_string(), client_id, &state, &room_name).await;
            }
        } else {
            handle_chat_message(text.to_string(), client_id, &state, &room_name).await;
        }
    }
}


/// Handles setting or updating a client's username and sends them the room history.
async fn handle_set_username(username: String, client_id: Uuid, state: &ChatState, room_name: &str) {
    let mut rooms = state.lock().await;
    let mut old_username = "anonymous".to_string();

    if let Some(room) = rooms.get_mut(room_name) {
        if let Some(client) = room.clients.get_mut(&client_id) {
            old_username = client.username.clone();
            client.username = username.clone();
            
            // Send room history to the user who just set their name
            for msg in &room.history {
                let parsed_msg = parse_message_for_display(msg);
                if client.sender.send(Message::Text(parsed_msg.into())).await.is_err() {
                    println!("Failed to send history to client {}", client_id);
                    return; // Early return if we can't send history
                }
            }
        }
    } else { return; } // Room doesn't exist, something is wrong

    println!("Client {} ({}) is now known as '{}' in room '{}'", client_id, old_username, &username, room_name);

    let join_msg = ServerMessage::UserJoined { username };
    broadcast_message(join_msg, &mut rooms, room_name, Some(client_id)).await;
}

/// Handles a regular chat message, adds it to history, and broadcasts it.
async fn handle_chat_message(content: String, client_id: Uuid, state: &ChatState, room_name: &str) {
    if content.trim().is_empty() { return; }
    
    let mut rooms = state.lock().await;

    let username = match rooms.get(room_name).and_then(|r| r.clients.get(&client_id)) {
        Some(client) => client.username.clone(),
        None => return, // Client not found
    };

    if username == "anonymous" {
        if let Some(client) = rooms.get_mut(room_name).and_then(|r| r.clients.get_mut(&client_id)) {
            let _ = client.sender.send(Message::Text("Please set a username with `/user <name>` before sending messages.".to_string().into())).await;
        }
        return;
    }

    println!("Message from {}({}): {}", &username, client_id, &content);
    let new_msg = ServerMessage::NewMessage { username, content };
    broadcast_message(new_msg, &mut rooms, room_name, Some(client_id)).await;
}

/// Broadcasts a message and adds it to the room's history.
async fn broadcast_message(
    message: ServerMessage,
    rooms: &mut MutexGuard<'_, HashMap<String, Room>>,
    room_name: &str,
    exclude_client_id: Option<Uuid>,
){
    if let Some(room) = rooms.get_mut(room_name) {
        // Add message to history, ensuring it doesn't exceed the max size.
        room.history.push_back(message.clone());
        if room.history.len() > MAX_HISTORY_SIZE {
            room.history.pop_front();
        }

        let parsed_message = parse_message_for_display(&message);
        for (id, client) in room.clients.iter_mut() {
            if exclude_client_id.map_or(false, |exclude_id| *id == exclude_id) {
                continue;
            }
            if client.sender.send(Message::Text(parsed_message.clone().into())).await.is_err() {
                println!("Failed to send parsed message to client {}", id);
            }
        }
    }
}

/// Converts a ServerMessage to a human-readable format for testing.
fn parse_message_for_display(message: &ServerMessage) -> String {
    match message {
        ServerMessage::NewMessage { username, content } => format!("[{}] {}", username, content),
        ServerMessage::UserJoined { username } => format!("--> {} joined the room", username),
        ServerMessage::UserLeft { username } => format!("<-- {} left the room", username),
    }
}

/// Removes a client from the state and announces their departure.
async fn cleanup_client(state: &ChatState, client_id: Uuid, room_name: &str) {
    let mut username = "anonymous".to_string();
    let mut should_broadcast = false;

    // First, remove the client and get their username
    {
        let mut rooms = state.lock().await;
        if let Some(room) = rooms.get_mut(room_name) {
            if let Some(client) = room.clients.remove(&client_id) {
                username = client.username;
                should_broadcast = username != "anonymous";
            }

            if room.clients.is_empty() {
                println!("Room '{}' is empty, removing it.", room_name);
                rooms.remove(room_name);
            }
        }
    } // First lock is released here

    // Now broadcast departure message with a fresh lock
    if should_broadcast {
        println!("Broadcasting leave message for {} from room '{}'", username, room_name);
        let left_msg = ServerMessage::UserLeft { username: username.clone() };
        let mut rooms_for_broadcast = state.lock().await;
        broadcast_message(left_msg, &mut rooms_for_broadcast, room_name, None).await;
    }

    println!("Client {} ({}) disconnected from room '{}'.", client_id, username, room_name);
}
