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

---

## 📝 Notes

- The server and clients must run on the same machine by default (or edit the IP/port in the code for LAN use).
- **Do not commit the `target/` directory**; it is ignored by `.gitignore`.
- `Cargo.lock` is included for reproducible builds.
- All dependencies are managed by Cargo.

---








Inspiration: https://www.youtube.com/watch?v=653rafFNBmA&t=5969s 
