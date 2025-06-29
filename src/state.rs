// src/state.rs

use axum::extract::ws::{Message, WebSocket};
use futures_util::stream::SplitSink;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use uuid::Uuid;

/// Represents a connected client, holding their username and the sender part of their WebSocket.
pub struct Client {
    pub username: String,
    pub sender: SplitSink<WebSocket, Message>,
}

/// The application's shared state, accessible from all request handlers.
///
/// It's a map of room names to another map of client IDs to `Client` structs.
/// `Arc<Mutex<...>>` ensures thread-safe access across asynchronous tasks.
pub type ChatState = Arc<Mutex<HashMap<String, HashMap<Uuid, Client>>>>;
