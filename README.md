# Real-Time Chat Server in Rust

A high-performance, concurrent, room-based chat server built from scratch in Rust to demonstrate advanced concepts in systems programming, including asynchronous I/O, shared-state concurrency, and WebSocket communication.

## Current Status (Phase 3 Complete)

The server is currently functional and supports the following features:

- **WebSocket-based Communication**: Uses axum and tokio-tungstenite for efficient real-time messaging
- **Multi-Client Support**: Can handle numerous simultaneous client connections
- **Room-Based Chat**: Clients can connect to specific, named rooms (e.g., `/ws/general`, `/ws/gaming`)
- **Message Broadcasting**: Messages sent to a room are broadcast to all other clients in that same room
- **Dynamic Room Management**: Rooms are created on-demand when the first user joins and are cleaned up automatically when the last user leaves
- **Asynchronous Architecture**: Built on the tokio runtime for non-blocking, scalable performance

## Features

- **WebSocket Support**: Real-time bidirectional communication using `tokio-tungstenite`
- **Async Runtime**: Built on Tokio for high-performance async operations
- **HTTP Server**: RESTful API endpoints using Axum framework
- **JSON Serialization**: Message handling with Serde for type-safe data exchange
- **Modern Rust**: Built with Rust 2024 edition
- **Concurrent Connections**: Handles multiple WebSocket clients simultaneously
- **Chat Rooms**: Support for multiple chat rooms with isolated messaging
- **Message Broadcasting**: Broadcasts messages to all clients in a room
- **Room Management**: Automatic room creation and cleanup
- **Hybrid History System**: In-memory caching (50 messages) + database persistence (1000+ messages)
- **Lazy Loading**: History loaded from database only when needed
- **Message Persistence**: All messages stored in PostgreSQL database

## Prerequisites

- Rust 1.70+ (for 2024 edition support)
- Cargo (Rust's package manager)

## Installation & How to Run

1. **Clone the Repository**:
```bash
git clone https://github.com/Ketankhunti/Rust-Chat-Server.git
cd Rust-Chat-Server
```

2. **Build the project**:
```bash
cargo build
```

3. **Run the Server**:
```bash
cargo run
```

The server will start and listen on `ws://localhost:3000`.

4. **Connect a Client**: Use a WebSocket client tool like websocat to connect. To join a room named "tech", for example:
```bash
websocat ws://localhost:3000/ws/tech
```

Open multiple terminals and connect to the same or different rooms to test the broadcasting functionality.

## Usage

### WebSocket Connection

Connect to a specific chat room:
```
ws://localhost:3000/ws/general
ws://localhost:3000/ws/random
ws://localhost:3000/ws/tech
```

Replace `{room}` with any room name you want to join. Messages sent in a room will be broadcast to all other clients in that same room.

### Message Format

Messages are displayed as:
```
[client-uuid]: your message here
```

### Testing with WebSocket Clients

You can test the server using:
- Browser WebSocket API
- WebSocket testing tools like wscat or Postman
- Custom WebSocket clients

Connect multiple clients to the same room to see real-time message broadcasting.

### Available Commands

- `/user <username>` - Set your username (required before sending messages)
- `/history` - Load full message history from the database (up to 1000 messages)

## Design & Architecture

### Core Framework
- **axum** is used as a lightweight web framework to handle the initial HTTP request and upgrade it to a WebSocket connection

### Asynchronous Runtime
- **tokio** powers the entire application, managing asynchronous tasks for each client connection and network event

### Concurrency Model
The project utilizes a **Shared-State Concurrency** model. A central `ChatState` data structure, protected by an `Arc<Mutex<...>>`, holds the state of all rooms and connected clients. This allows each independent client task to safely access and modify the shared data.

### Lifecycle Management
Each client connection is handled in its own spawned task. The `tokio::select!` macro is used to gracefully manage the connection's lifecycle, ensuring proper cleanup (removing the client from the state) when a connection is closed.

## Dependencies

- **tokio**: Async runtime with full features
- **tokio-tungstenite**: WebSocket implementation
- **futures-util**: Future utilities
- **axum**: Web framework for building HTTP APIs (with WebSocket support)
- **serde**: Serialization/deserialization framework
- **serde_json**: JSON support for Serde
- **uuid**: Unique identifier generation for clients

## Project Structure

```
chat_server/
├── src/
│   ├── main.rs         # Entry point, sets up server
│   ├── state.rs        # Defines ChatState and related structs
│   ├── models.rs       # Defines our JSON message structures (ChatMessage, etc.)
│   └── websocket.rs    # Contains all our handlers (websocket_handler, handle_socket, etc.)
├── Cargo.toml          # Project dependencies and metadata
├── Cargo.lock          # Locked dependency versions
└── .gitignore         # Git ignore rules
```

## Implementation Status

### ✅ Phase 1: Basic WebSocket Echo Server (COMPLETED)
- WebSocket server listening on port 3000
- Concurrent client handling with async/await
- Message echoing functionality
- Proper connection lifecycle management
- Support for multiple simultaneous connections

### ✅ Phase 2: Chat Room Functionality (COMPLETED)
- Multiple chat room support with dynamic room creation
- Message broadcasting to all clients in a room
- Room-based message isolation
- Automatic room cleanup when empty
- Client identification with UUIDs
- Proper connection lifecycle management per room

### ✅ Phase 3: Advanced Features (COMPLETED)
- Full room-based chat system
- Dynamic room management
- Concurrent client handling
- WebSocket communication
- Shared-state concurrency model
- Message persistence with PostgreSQL database
- Hybrid history system with in-memory caching

### 🔄 Phase 4: Usernames & Structured Messages (COMPLETED)
- [✓] Implement serde for JSON message passing
- [✓] Allow users to set a username upon connecting
- [✓] Broadcast UserJoined and UserLeft notifications to the room
- [✓] Structure all messages as JSON objects (e.g., `{ "type": "NewMessage", "username": "Alice", "content": "Hello!" }`)
- [✓] Message persistence and history loading

## Future Development (Phase 4 and Beyond)

The next phase will focus on adding richer application-level features.

### Bonus Features (To Extend the Project)

- **Message Persistence**: Integrate a simple database (like SQLite with sqlx) or file-based logging to save and load room chat history
- **Private Messaging**: Implement a mechanism for one-to-one messaging between users
- **Authentication**: Add a basic authentication layer (e.g., token-based) to protect rooms
- **Modular Refactoring**: Reorganize the codebase into separate modules (`state.rs`, `models.rs`, `websocket.rs`) for better maintainability
- **Frontend Client**: Build a simple web-based frontend using HTML, CSS, and JavaScript to interact with the server

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Roadmap

- [✓] WebSocket server implementation
- [✓] Concurrent client handling
- [✓] Message echoing
- [✓] Chat room functionality
- [✓] Message broadcasting
- [✓] Room management
- [✓] User management with usernames
- [✓] JSON message structuring
- [✓] Message persistence
- [ ] Authentication system
- [ ] REST API endpoints
- [ ] Client examples
- [ ] Docker support
- [ ] Configuration management
- [ ] Logging and monitoring 