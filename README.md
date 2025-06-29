# Rust Chat Server

A real-time chat server built with Rust, featuring WebSocket support and modern async/await patterns.

## Features

- **WebSocket Support**: Real-time bidirectional communication using `tokio-tungstenite`
- **Async Runtime**: Built on Tokio for high-performance async operations
- **HTTP Server**: RESTful API endpoints using Axum framework
- **JSON Serialization**: Message handling with Serde for type-safe data exchange
- **Modern Rust**: Built with Rust 2024 edition
- **Concurrent Connections**: Handles multiple WebSocket clients simultaneously
- **Echo Server**: Currently implements message echoing functionality

## Prerequisites

- Rust 1.70+ (for 2024 edition support)
- Cargo (Rust's package manager)

## Installation

1. Clone the repository:
```bash
git clone https://github.com/Ketankhunti/Rust-Chat-Server.git
cd Rust-Chat-Server
```

2. Build the project:
```bash
cargo build
```

3. Run the server:
```bash
cargo run
```

The server will start on `ws://0.0.0.0:3000/ws`

## Usage

### WebSocket Connection

Connect to the WebSocket endpoint:
```
ws://localhost:3000/ws
```

The server will echo back any text messages you send. Send a `Close` frame to disconnect.

### Testing with WebSocket Clients

You can test the server using:
- Browser WebSocket API
- WebSocket testing tools like wscat or Postman
- Custom WebSocket clients

## Dependencies

- **tokio**: Async runtime with full features
- **tokio-tungstenite**: WebSocket implementation
- **futures-util**: Future utilities
- **axum**: Web framework for building HTTP APIs (with WebSocket support)
- **serde**: Serialization/deserialization framework
- **serde_json**: JSON support for Serde

## Project Structure

```
chat_server/
├── src/
│   └── main.rs          # Main application entry point with WebSocket server
├── Cargo.toml           # Project dependencies and metadata
├── Cargo.lock           # Locked dependency versions
└── .gitignore          # Git ignore rules
```

## Implementation Status

### ✅ Phase 1: Basic WebSocket Echo Server (COMPLETED)
- WebSocket server listening on port 3000
- Concurrent client handling with async/await
- Message echoing functionality
- Proper connection lifecycle management
- Support for multiple simultaneous connections

### 🔄 Phase 2: Chat Room Functionality (PLANNED)
- Chat room management
- Message broadcasting to all connected clients
- User identification and management
- Message persistence

### 🔄 Phase 3: Advanced Features (PLANNED)
- User authentication
- REST API endpoints
- Message history
- Private messaging

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
- [ ] Chat room functionality
- [ ] User management
- [ ] Message broadcasting
- [ ] Message persistence
- [ ] Authentication system
- [ ] REST API endpoints
- [ ] Client examples
- [ ] Docker support
- [ ] Configuration management
- [ ] Logging and monitoring 