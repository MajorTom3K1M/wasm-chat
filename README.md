# Wasm Chat Application

A lightweight chat application that allows users to join chats, send messages, and receive messages in real time.

## Key Features

- **Join Chat**: Seamlessly join chat rooms.
- **Real-Time Messaging**: Send and receive messages instantly.
- **PubNub Integration**: Leverages PubNub for efficient real-time communication.

## Technologies Used

- **[Rust](https://www.rust-lang.org/)**: Version 1.73, for a robust and high-performance backend.
- **[Yew.rs](https://yew.rs/)**: Frontend framework for building web applications in Rust.
- **[PubNub](https://www.pubnub.com/)**: Real-time communication framework.

## Prerequisites

Before running the project, ensure you have the following installed:

- Rust (version 1.73 or higher)
- Cargo (Rust package manager)
- Trunk (for building and serving the Yew frontend)
- Docker Compose (optional, for running the Dockerized version)

## Configuration

### PubNub Keys

- The project requires `publish_key` and `subscribe_key` to interact with PubNub.
- These keys are defined in the `services.rs` file.
- Update the keys with your own PubNub credentials before running the project:
  ```rust
    let init = Initialize {
        publish_key: "publish_key".to_owned(),
        subscribe_key: "subscribe_key".to_owned(),
        heartbeat_interval: 6
    };
  ```

## Getting Started

### Run Locally

1. Clone the repository:
   ```bash
   git clone https://github.com/MajorTom3K1M/wasm-chat.git
   cd chat-application
   ```

2. Fetch dependencies:
   ```bash
   cargo fetch
   ```

3. Install Trunk (version 0.16.0):
   ```bash
   cargo install trunk --version 0.16.0
   ```

4. Start the development server:
   ```bash
   trunk serve
   ```

### Run with Docker Compose

1. Clone the repository:
   ```bash
   git clone https://github.com/MajorTom3K1M/wasm-chat.git
   cd chat-application
   ```

2. Update `publish_key` and `subscribe_key` in `services.rs` with your PubNub credentials.

3. Start the application using Docker Compose:
   ```bash
   docker-compose up
   ```

## New Version
![Modernize Chat Image](https://github.com/MajorTom3K1M/wasm-chat/blob/main/screenshot/new-screenshot-1.png)

## Old Vesrion
![Chat Image](https://github.com/MajorTom3K1M/wasm-chat/blob/main/screenshot/screenshot-1.png)