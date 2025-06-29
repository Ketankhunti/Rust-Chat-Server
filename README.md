# Rust Chat Server

A real-time chat server built with Rust, featuring WebSocket support and modern async/await patterns.

## Features

- **WebSocket Support**: Real-time bidirectional communication using `tokio-tungstenite`
- **Async Runtime**: Built on Tokio for high-performance async operations
- **HTTP Server**: RESTful API endpoints using Axum framework
- **JSON Serialization**: Message handling with Serde for type-safe data exchange
- **Modern Rust**: Built with Rust 2024 edition

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

## Dependencies

- **tokio**: Async runtime with full features
- **tokio-tungstenite**: WebSocket implementation
- **futures-util**: Future utilities
- **axum**: Web framework for building HTTP APIs
- **serde**: Serialization/deserialization framework
- **serde_json**: JSON support for Serde

## Project Structure

```
chat_server/
├── src/
│   └── main.rs          # Main application entry point
├── Cargo.toml           # Project dependencies and metadata
├── Cargo.lock           # Locked dependency versions
└── .gitignore          # Git ignore rules
```

## Development

The project is currently in early development. The main.rs file contains a basic "Hello, world!" implementation that will be expanded to include:

- WebSocket connection handling
- Chat room management
- Message broadcasting
- User authentication
- REST API endpoints

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Roadmap

- [ ] WebSocket server implementation
- [ ] Chat room functionality
- [ ] User management
- [ ] Message persistence
- [ ] Authentication system
- [ ] REST API endpoints
- [ ] Client examples
- [ ] Docker support
- [ ] Configuration management
- [ ] Logging and monitoring 