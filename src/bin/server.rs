
//setting up tokio for TCP server with broadcast messaging, asynchronous handling of client connections,
// and message history management

use tokio::{
    net::{TcpListener, TcpStream},
    sync::broadcast,
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
};

// Serde: serializing and deserializing (JSON handling)
//Chrono: timestamp for when a user joins the chat room 
//Arc: good for shared ownership of data across threads
//Mutex: allows mutable access to shared data in a thread-safe manner
//VecDeque: a double-ended queue that allows efficient push and pop operations from both ends
use serde::{Serialize, Deserialize};
use chrono::Local;
use std::error::Error;
use std::collections::VecDeque;
use tokio::sync::Mutex;
use std::sync::Arc;


// Define the structure of a chat message below 

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatMessage {
    username: String,
    content: String,
    timestamp: String,
    message_type: MessageType,
}


// Define the type of messages that can be sent

#[derive(Debug, Clone, Serialize, Deserialize)]
enum MessageType {
    UserMessage,
    SystemNotification,
}



//#tokio main creates a pool of asynchronous threads for message handling while starting up the server
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1:8082").await?; //bind the server to the specified address and port


    // Create a shared history buffer with a maximum capacity of 20 messages
    let history = Arc::new(Mutex::new(VecDeque::with_capacity(20)));


    //Output in the command line for server startup (yes I know it is slightly off-centered)

    println!("################################################################################");
    println!("#                                                                              #");
    println!("#   ██████   ███    ██  ███      ██  ███    ██  ███████                        #");
    println!("#   ██   ██  ████   ██   ██      ██  ████   ██  ██                             #");
    println!("#   ██    ██ ██ ██  ██   ██      ██  ██ ██  ██  █████                          #");
    println!("#   ██    ██ ██  ██ ██   ██      ██  ██  ██ ██  ██                             #");
    println!("#    ██████  ██   ████   ███████ ██  ██   ████  ███████                        #");
    println!("#                                                                              #");
    println!("#                        🚀  SERVER ONLINE  🚀                                 #");
    println!("#                   Listening: 127.0.0.1:8082                                  #");
    println!("#                [CTRL+C] to disengage hyperdrive                              #");
    println!("#                                                                              #");
    println!("################################################################################");


    //tx used for broadcasting messages to all connected clients
    //rx used for receiving messages from the broadcast channel
    //broadcast channel with a buffer size of 200 messages (meaning that it hold up to 200 messages before blocking) 
    

    let (tx, _) = broadcast::channel::<String>(200);


    //shutdown_signal is used to gracefully shut down the server when Ctrl+C is pressed
    //tokio::signal::ctrl_c() creates a future that resolves when the user presses Ctrl+C
    //tokio::pin! is used to pin the shutdown_signal future so it can be used in a select statement

    let shutdown_signal = tokio::signal::ctrl_c();
    tokio::pin!(shutdown_signal);



    //this loop accepts new connections and spawns a new task for each connection
    //tokio::select! is used to wait for either a new connection or the shutdown signal
    //when a new connection is accepted, it prints the connection details and spawns a new task to handle the connection
    //the task handles reading messages from the client, broadcasting them to all connected clients, and sending the message history to the new client
    //when the shutdown signal is received, it sends a shutdown message to all clients and breaks the loop to shut down the server gracefully
    loop {
        tokio::select! {
            Ok((socket, addr)) = listener.accept() => {
                println!("┌─[{}] New connection", Local::now().format("%D:%H:%M:%S"));
                println!("└─ Address: {}", addr);

                let tx = tx.clone();
                let rx = tx.subscribe();
                let history = history.clone();

                tokio::spawn(async move {
                    handle_connection(socket, tx, rx, history).await
                });
            }

            _ = &mut shutdown_signal => {
                println!("\n🛑 Ctrl+C received. Starting graceful shutdown…");

                let shutdown_msg = ChatMessage {
                    username: "System".to_string(),
                    content: "Server is shutting down...".to_string(),
                    timestamp: Local::now().format("%D:%H:%M:%S").to_string(),
                    message_type: MessageType::SystemNotification,
                };

                let shutdown_json = match serde_json::to_string(&shutdown_msg) {
                    Ok(j) => j,
                    Err(e) => {
                        eprintln!("[ERROR] failed to serialize shutdown message: {}", e);
                        break;
                    }
                };
                let _ = tx.send(shutdown_json);

                break;
            }
        }
    }

    drop(tx);

    println!("✅ Server has shut down gracefully.");
    Ok(())
}



// This function handles a single client connection asynchronously 

async fn handle_connection(
    mut socket: TcpStream,
    tx: broadcast::Sender<String>,
    mut rx: broadcast::Receiver<String>,
    history: Arc<Mutex<VecDeque<ChatMessage>>>,
) {
    let (reader, mut writer) = socket.split();
    let mut reader = BufReader::new(reader);
    let mut username = String::new();

    // 1. Read the username (gracefully bail on error)
    if let Err(e) = reader.read_line(&mut username).await {
        eprintln!("[ERROR] failed to read username: {}", e);
        return;
    }
    let username = username.trim().to_string();

    // Helper to send broadcast without panicking (such as with unwrap) 
    let try_send = |tx: &broadcast::Sender<String>, msg: String| {
        if let Err(e) = tx.send(msg) {
            eprintln!("[WARN] broadcast send failed: {}", e);
        }
    };

    // 2. Announce new user arrival 
    let join_msg = ChatMessage {
        username: username.clone(),
        content: "has landed".into(),
        timestamp: Local::now().format("%H:%M:%S").to_string(),
        message_type: MessageType::SystemNotification,
    };
    let join_json = match serde_json::to_string(&join_msg) {
        Ok(j) => j,
        Err(e) => {
            eprintln!("[ERROR] failed to serialize join message: {}", e);
            return;
        }
    };
    try_send(&tx, join_json.clone());

    // 2 continued.... Send message history to the new client so they can catch up 
    {
        let history = history.lock().await;
        for msg in history.iter() {
            if let Ok(json) = serde_json::to_string(msg) {
                let _ = writer.write_all(json.as_bytes()).await;
                let _ = writer.write_all(b"\n").await;
            }
        }
        let _ = writer.flush().await;
    }

    // 3. Main loop: read client messages & forward broadcasts
    let mut line = String::new();
    loop {
        tokio::select! {
            // A) Incoming from client
            result = reader.read_line(&mut line) => {
                match result {
                    Ok(0) => break, // client disconnected
                    Ok(_) => {
                        let trimmed = line.trim();
                        if !trimmed.is_empty() {
                            let msg = ChatMessage {
                                username: username.clone(),
                                content: trimmed.to_string(),
                                timestamp: Local::now().format("%D:%H:%M:%S").to_string(),
                                message_type: MessageType::UserMessage,
                            };
                            let json = match serde_json::to_string(&msg) {
                                Ok(j) => j,
                                Err(e) => {
                                    eprintln!("[ERROR] failed to serialize message: {}", e);
                                    line.clear();
                                    continue;
                                }
                            };
                            // Add to history so it remains dynamic 
                            {
                                let mut history = history.lock().await;
                                if history.len() == 20 {
                                    history.pop_front();
                                }
                                history.push_back(msg);
                            }
                            if let Err(e) = tx.send(json) {
                                eprintln!("[WARN] broadcast send failed: {}", e);
                            }
                        }
                        line.clear();
                    }
                    Err(e) => {
                        eprintln!("[ERROR] failed to read from {}: {}", username, e);
                        break;
                    }
                }
            }

            // B) Incoming broadcast to send to this client
            result = rx.recv() => {
                match result {
                    Ok(msg) => {
                        if let Err(e) = writer.write_all(msg.as_bytes()).await {
                            eprintln!("[ERROR] writing to {}: {}", username, e); //handles errors when writing to the client
                            break;
                        }
                        if let Err(e) = writer.write_all(b"\n").await {
                            eprintln!("[ERROR] writing newline to {}: {}", username, e); //handles errors when writing a newline to the client
                            break;
                        }
                        if let Err(e) = writer.flush().await {
                            eprintln!("[ERROR] flushing to {}: {}", username, e); //handles errors when flushing the writer to the client
                            break;
                        }
                    }

                    //broadcasts errors 
                    Err(broadcast::error::RecvError::Closed) => {
                        break;
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        eprintln!("[WARN] {} missed {} messages", username, n);
                        continue;
                    }
                }
            }
        }
    }

    // 4. Announce departure
    let leave_msg = ChatMessage {
        username: username.clone(),
        content: "has blasted off".into(),
        timestamp: Local::now().format("%D:%H:%M:%S").to_string(),
        message_type: MessageType::SystemNotification,
    };
    let leave_json = match serde_json::to_string(&leave_msg) {
        Ok(j) => j,
        Err(e) => {
            eprintln!("[ERROR] failed to serialize leave message: {}", e);
            return;
        }
    };
    try_send(&tx, leave_json);
    println!("└─[{}] {} disconnected", Local::now().format("%D:%H:%M:%S"), username);
}