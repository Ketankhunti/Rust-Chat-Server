// src/state.rs

use crate::models::ServerMessage;
use axum::extract::ws::{Message, WebSocket};
use futures_util::stream::SplitSink;
use sqlx::PgPool; // For PostgreSQL
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

/// Represents a connected client, holding their username and the sender part of their WebSocket.
pub struct Client {
    pub username: String,
    pub sender: SplitSink<WebSocket, Message>,
}

/// Represents a chat room, containing all connected clients and a cached history of recent messages.
#[derive(Default)]
pub struct Room {
    pub clients: HashMap<Uuid, Client>,
    pub history: VecDeque<ServerMessage>,
}

// Configuration constants for the hybrid approach
pub const IN_MEMORY_CACHE_SIZE: usize = 50;  // Keep last 50 messages in memory
pub const MAX_HISTORY_SIZE: usize = 1000;    // Maximum messages to load from DB

/// The application's shared state, accessible from all request handlers.
/// This struct is created once in `main.rs` and shared across all connections via Axum's state management.
#[derive(Clone)]
pub struct ChatState {
    pub rooms: Arc<Mutex<HashMap<String, Room>>>,
    pub db_pool: PgPool,
}
