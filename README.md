# 🚀 Rust Retro TCP Chat App 🚀 

A space/retro-themed, async TCP chat server and client with a terminal UI, built in Rust using Tokio, Chrono, Serde and Cursive.

---

## ✨ Features

- Real-time chat with multiple clients
- Fun retro terminal UI (Cursive)
- Emoji and ASCII art support
- Message history for new arrivals
- Simple commands: `/help`, `/clear`, `/quit`, `/funface`
- Colorful usernames
- Graceful shutdown process

Emojify Example:

![Some Emojis](Screenshot%202025-05-28%20124048.png) 

---

## 🛠️ Prerequisites

- [Rust & Cargo](https://rustup.rs/) (install with `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- A terminal that supports Unicode and colors

---

## 🚦 Getting Started

Clone the repo and build:

```bash
git clone https://github.com/5quidL0rd/Rust_TCP_Chat_App.git
cd Rust_TCP_Chat_App
cargo build --release
```

---

## 🖥️ Running the Server

Start the chat server (listens on `127.0.0.1:8082` and hosted locally):

```bash
cargo run --bin server
```

---

## 💬 Running the Client

Open a new terminal for each client.  
Start a client with your chosen username:

```bash
cargo run --bin client "name" 
```

Example:

```bash
cargo run --bin client "Bobrovsky"
```

---

## 💡 Client Commands

- `/help`    — Show help
- `/clear`   — Clear chat window
- `/quit`    — Exit chat
- `/funface` — Show ASCII art

Example:

![ASCII art](Screenshot%202025-05-28%20123715.png) 

---



---

## 🕑 Message History

- **Automatic Message History:**  
  When a new client joins the chat, they automatically receive recent message history so they can catch up on the conversation. The server maintains a buffer of recent messages and sends them to each new user upon connection. This ensures everyone is up to speed, even if they join late.

- **Configurable Buffer:**  
  The number of messages stored in history can be easily adjusted in the server code, allowing you to control how much context new users receive.


  ![Message History](Screenshot%202025-05-28%20123433.png) 

---

## 🛑 Graceful Shutdown & Robust Error Handling

- **Graceful Shutdown:**  
  Both the server and client handle shutdown signals (such as `Ctrl+C`) cleanly. When you stop the server, all connected clients are notified and the server closes all connections without data loss or corruption. The client also exits cleanly when you use `/quit` or close the terminal.

- **Error Handling:**  
  The application uses Rust’s robust error handling (`Result`, `?`, and custom messages) to manage network failures, invalid input, and unexpected disconnects. If a client loses connection or sends malformed data, the error is logged and the app continues running for other users.  
  User-friendly error messages are shown in the client UI for common issues (e.g., connection refused, invalid username).


  Example:

  ![Server graceful shutdown](Screenshot%202025-05-28%20122849.png)

  ![Chat App shutdown notif](Screenshot%202025-05-28%20122830.png) 

---

## 📝 Notes

- The server and clients must run on the same machine by default (or edit the IP/port in the code for LAN use).
- **Do not commit the `target/` directory**; it is ignored by `.gitignore`.
- `Cargo.lock` is included for reproducible builds.
- All dependencies are managed by Cargo.

---


## 🎨 Screenshots

![Chat UI Example](Screenshot%202025-05-28%20120507.png) 

[![Chat UI Example](Screenshot%202025-05-28%20120244.png)]



---

## 🎥 Inspiration

This project was inspired by the following video:[ https://www.youtube.com/watch?v=653rafFNBmA&t=5969s](https://www.youtube.com/watch?v=653rafFNBmA&t=5969s)
Many thanks to the creator for the excellent walkthrough and ideas!

---
