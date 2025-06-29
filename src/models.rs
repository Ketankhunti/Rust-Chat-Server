// src/models.rs

use serde::{Deserialize, Serialize};

/// A message sent from a client to the server.
/// Deserialized from incoming JSON text.
#[derive(Deserialize, Debug)]
#[serde(tag = "type")] // Use a 'type' field to determine which variant it is
pub enum ClientMessage {
    SetUsername { username: String },
    Message { content: String },
}

/// A message sent from the server to a client.
/// Serialized into JSON text for sending.
#[derive(Serialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum ServerMessage {
    UserJoined { username: String },
    UserLeft { username: String },
    NewMessage { username: String, content: String },
}
