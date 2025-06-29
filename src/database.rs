// src/database.rs

use crate::models::ServerMessage;
use serde_json;
use sqlx::{postgres::PgPool, Row};
use std::collections::VecDeque;

// IMPORTANT: Replace with your actual PostgreSQL connection details.
const DB_URL: &str = "postgres://postgres:postgres@localhost:5433/chat_db";
const MAX_HISTORY_SIZE: usize = 10;

/// Initializes the database and creates the `messages` table if it doesn't exist.
pub async fn setup_database() -> Result<PgPool, sqlx::Error> {
    let pool = PgPool::connect(DB_URL).await?;

    // Use PostgreSQL specific syntax
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS messages (
            id SERIAL PRIMARY KEY,
            room TEXT NOT NULL,
            message JSONB NOT NULL,
            timestamp TIMESTAMPTZ DEFAULT NOW()
        )",
    )
    .execute(&pool)
    .await?;

    println!("PostgreSQL Database setup complete.");
    Ok(pool)
}

/// Saves a message to the database.
pub async fn save_message(pool: &PgPool, room_name: &str, message: &ServerMessage) {
    let message_json = match serde_json::to_value(message) {
        Ok(json) => json,
        Err(e) => {
            eprintln!("Failed to serialize message for DB: {}", e);
            return;
        }
    };

    // Use PostgreSQL's $1, $2 placeholder syntax
    if let Err(e) = sqlx::query("INSERT INTO messages (room, message) VALUES ($1, $2)")
        .bind(room_name)
        .bind(&message_json)
        .execute(pool)
        .await
    {
        eprintln!("Failed to save message to DB: {}", e);
    }
}

/// Loads the last N messages for a specific room from the database.
pub async fn load_history(pool: &PgPool, room_name: &str, limit: usize) -> VecDeque<ServerMessage> {
    let query = format!(
        "SELECT message FROM messages WHERE room = $1 ORDER BY timestamp DESC LIMIT {}",
        limit
    );

    let rows = match sqlx::query(&query).bind(room_name).fetch_all(pool).await {
        Ok(rows) => rows,
        Err(e) => {
            eprintln!("Failed to load history from DB: {}", e);
            return VecDeque::new();
        }
    };

    let mut history: VecDeque<ServerMessage> = VecDeque::with_capacity(limit);
    for row in rows.into_iter().rev() { // Reverse to get chronological order
        if let Ok(message_json) = row.try_get::<serde_json::Value, _>("message") {
            if let Ok(message) = serde_json::from_value(message_json) {
                history.push_back(message);
            }
        }
    }
    history
}

/// Loads paginated history for a specific room from the database.
pub async fn load_history_paginated(
    pool: &PgPool, 
    room_name: &str, 
    page: i32, 
    page_size: i32
) -> VecDeque<ServerMessage> {
    let offset = (page - 1) * page_size;
    let query = format!(
        "SELECT message FROM messages WHERE room = $1 ORDER BY timestamp DESC LIMIT {} OFFSET {}",
        page_size, offset
    );

    let rows = match sqlx::query(&query).bind(room_name).fetch_all(pool).await {
        Ok(rows) => rows,
        Err(e) => {
            eprintln!("Failed to load paginated history from DB: {}", e);
            return VecDeque::new();
        }
    };

    let mut history: VecDeque<ServerMessage> = VecDeque::with_capacity(page_size as usize);
    for row in rows.into_iter().rev() { // Reverse to get chronological order
        if let Ok(message_json) = row.try_get::<serde_json::Value, _>("message") {
            if let Ok(message) = serde_json::from_value(message_json) {
                history.push_back(message);
            }
        }
    }
    history
}

/// Gets the total count of messages for a specific room.
pub async fn get_message_count(pool: &PgPool, room_name: &str) -> i64 {
    match sqlx::query("SELECT COUNT(*) FROM messages WHERE room = $1")
        .bind(room_name)
        .fetch_one(pool)
        .await
    {
        Ok(row) => row.get(0),
        Err(e) => {
            eprintln!("Failed to get message count from DB: {}", e);
            0
        }
    }
}
