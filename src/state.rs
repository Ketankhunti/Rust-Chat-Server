// src/state.rs

use crate::models::ServerMessage;
use axum::extract::ws::{Message, WebSocket};
use futures_util::stream::SplitSink;
use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};
use tokio::sync::Mutex;
use uuid::Uuid;

/// Represents a connected client, holding their username and the sender part of their WebSocket.
pub struct Client {
    pub username: String,
    pub sender: SplitSink<WebSocket, Message>,
}

/// Represents a chat room, containing all connected clients and a history of recent messages.
#[derive(Default)]
pub struct Room {
    pub clients: HashMap<Uuid, Client>,
    pub history: VecDeque<ServerMessage>,
}

/// The application's shared state, accessible from all request handlers.
///
/// It's a map of room names to `Room` structs.
/// `Arc<Mutex<...>>` ensures thread-safe access across asynchronous tasks.
pub type ChatState = Arc<Mutex<HashMap<String, Room>>>;
