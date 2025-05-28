// Importing from cursive library to create a UI 
use cursive::{
    align::HAlign, event::Key, theme::{BaseColor, BorderStyle, Color, ColorStyle, Palette, PaletteColor, Theme}, traits::*, utils::markup::StyledString, views::{Dialog, DummyView, EditView, LinearLayout, Panel, ScrollView, TextView}, Cursive // Main Cursive application object
};

// Importing Serde for serialization and deserialization for JSON handling 
use serde::{Deserialize, Serialize};

//imporitng models for error handling and shared ownership of data 
use std::{env, error::Error, sync::Arc};

// Importing Tokio async utilities
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader}, 
    net::TcpStream, 
    sync::Mutex, 
};


// Chrono for date and time 
use chrono::Local;

// Structutre of a chat message 
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatMessage {
    username: String, 
    content: String, 
    timestamp: String, 
    message_type: MessageType, 
}

// Enum to represent different types of messages
#[derive(Debug, Clone, Serialize, Deserialize)]
enum MessageType {
    UserMessage, 
    SystemNotification, 
}

// Main asynchronous function to run the chat client
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Creates username from command line argument, and orders user to give one if they fail to do so
    let username = env::args()
        .nth(1) 
        .expect("Please provide a username as an argument"); 

    // UI framework initialized 
    let mut siv = cursive::default();
    siv.set_theme(create_space_theme()); //"space" theme for chat room 

    // Header of the chat room 
    let header = TextView::new(format!(r#"== CHATBOX == Chatterer: {} == {} =="#,
        username, 
        Local::now().format("%D:%H:%M:%S") 
    ))
    .style(Color::Light(BaseColor::Cyan)) 
    .h_align(HAlign::Center); 

    // Message area that is scrollable 
    let messages = TextView::new("") 
        .with_name("messages") 
        .min_height(50) 
        .scrollable(); 



    // Setting up the scroll view for messages

    let messages = ScrollView::new(messages)
        .scroll_strategy(cursive::view::ScrollStrategy::StickToTop) // Keep the scroll at the bottom 
        .min_width(30) 
        .full_width(); 

    // Creating an input area for typing messages
    let input = EditView::new()
        .on_submit(move |s, text| send_message(s, text.to_string())) 
        .with_name("input") 
        .min_width(50) 
        .max_height(5) 
        .full_width(); 

    // Creating help text for user commands
    let help_text = TextView::new("Ctrl+C:quit | Enter:send | Commands: /help, /clear, /quit, /funface")
        .style(Color::Dark(BaseColor::Green));

    // Creating the main layout of the chat application
    let layout = LinearLayout::vertical()
        .child(Panel::new(header))
        .child(
            Dialog::around(messages) 
                .title("Chattering") // Title 
                .title_position(HAlign::Center) // Center-align 
                .full_width()
        )
        .child( 
            Dialog::around(input) 
                .title("Chit Chat") 
                .title_position(HAlign::Left) 
                .full_width()
        )
        .child(Panel::new(help_text).full_width()); 

    // Wrapping layout for centering
    let centered_layout = LinearLayout::horizontal()
        .child(DummyView.full_width()) 
        .child(layout)
        .child(DummyView.full_width());

    // Adding the centered layout to the Cursive root
    siv.add_fullscreen_layer(centered_layout);

    // Adding global key bindings
    siv.add_global_callback(Key::Esc, |s| s.quit()); 
    siv.add_global_callback('/', |s| {
        s.call_on_name("input", |view: &mut EditView| {
            view.set_content("/"); 
        });
    });

    // Establishing a connection to the chat server, inbound to port 8082
    // This is where the client connects to the server
    let stream = TcpStream::connect("127.0.0.1:8082").await?;
    let (reader, mut writer) = stream.into_split(); 

    writer.write_all(format!("{}\n", username).as_bytes()).await?; 

    let writer = Arc::new(Mutex::new(writer)); 
    let writer_clone = Arc::clone(&writer); // Clone writer for later use
    siv.set_user_data(writer); // Store writer in the Cursive app data

    let reader = BufReader::new(reader); // Create a buffered reader for the stream
    let mut lines = reader.lines(); // Create an iterator over the lines of the stream
    let sink = siv.cb_sink().clone(); // Get a callback sink to update the UI

    // Spawn an async task to handle incoming messages
    tokio::spawn(async move {
        while let Ok(Some(line)) = lines.next_line().await {
            if let Ok(msg) = serde_json::from_str::<ChatMessage>(&line) {
                // Format incoming message based on type
                let formatted_msg = match msg.message_type {
    MessageType::UserMessage => {
        let mut styled = StyledString::plain(format!("┌─[{}]\n└─ ", msg.timestamp));
        styled.append_styled(msg.username.clone(), color_for_username(&msg.username));
        styled.append_plain(format!(" --> {}\n", msg.content));
        styled
    }
    MessageType::SystemNotification => {
        let mut styled = StyledString::plain("\n[");
        styled.append_styled(msg.username.clone(), color_for_username(&msg.username));
        styled.append_plain(format!(" {}]\n", msg.content));
        styled
    }
};
                // Update UI with the new message
                if sink.send(Box::new(move |siv: &mut Cursive| {
                    siv.call_on_name("messages", |view: &mut TextView| {
                        view.append(formatted_msg); // Append the message
                    });
                })).is_err() {
                    break; 
                }
            }
        }
    });

    siv.run(); // Run cursive events 
    let _ = writer_clone.lock().await.shutdown().await; 
    Ok(()) 
}

// Function to handle sending messages
fn send_message(siv: &mut Cursive, msg: String) {
    if msg.is_empty() { 
        return
    }

    // extra commands 
    match msg.as_str() {
        "/help" => {
            siv.call_on_name("messages", |view: &mut TextView| {
                view.append("\n=== Commands ===\n/help - Show this help\n/clear - Clear messages\n/quit - Exit chat\n\n");
            });
            siv.call_on_name("input", |view: &mut EditView| {
                view.set_content("");
            });
            return;
        }
        "/clear" => {
            siv.call_on_name("messages", |view: &mut TextView| {
                view.set_content(""); // Clear messages
            });
            siv.call_on_name("input", |view: &mut EditView| {
                view.set_content(""); // Clear input
            });
            return;
        }
        "/quit" => {
            siv.quit();
            return;
}
        "/funface" => {
            siv.call_on_name("messages", |view: &mut TextView| {
                view.append(
"                        $$$$$$$$$$$$$$$$$$$$
                       $$$$$$$$$$$$$$$$$$$$$$$$$$$
                    $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$         $$   $$$$$
    $$$$$$        $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$       $$$$$$$$$$
 $$ $$$$$$      $$$$$$$$$$    $$$$$$$$$$$$$    $$$$$$$$$$       $$$$$$$$
 $$$$$$$$$     $$$$$$$$$$      $$$$$$$$$$$      $$$$$$$$$$$    $$$$$$$$
   $$$$$$$    $$$$$$$$$$$      $$$$$$$$$$$      $$$$$$$$$$$$$$$$$$$$$$$
   $$$$$$$$$$$$$$$$$$$$$$$    $$$$$$$$$$$$$    $$$$$$$$$$$$$$  $$$$$$
    $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$     $$$$
     $$$   $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$     $$$$$
    $$$$   $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$       $$$$
    $$$    $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$ $$$$$$$$$$$$$$$$$
   $$$$$$$$$$$$$  $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$   $$$$$$$$$$$$$$$$$$
   $$$$$$$$$$$$$   $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$     $$$$$$$$$$$$
  $$$$       $$$$    $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$      $$$$
             $$$$$     $$$$$$$$$$$$$$$$$$$$$$$$$         $$$
               $$$$          $$$$$$$$$$$$$$$           $$$$
                $$$$$                                $$$$$
                 $$$$$$      $$$$$$$$$$$$$$        $$$$$
                   $$$$$$$$     $$$$$$$$$$$$$   $$$$$$$
                      $$$$$$$$$$$  $$$$$$$$$$$$$$$$$
                         $$$$$$$$$$$$$$$$$$$$$$
                                 $$$$$$$$$$$$$$$
                                     $$$$$$$$$$$$
                                      $$$$$$$$$$$
                                       $$$$$$$$\n"
                ); // Insert this fun guy 
            });
            siv.call_on_name("input", |view: &mut EditView| {
                view.set_content("");
            });
        }
        _ => {}
    }



    let msg = emojify(&msg);



    // Send the message to the server
    // Convert the message to a ChatMessage struct
    let writer = siv.user_data::<Arc<Mutex<tokio::net::tcp::OwnedWriteHalf>>>().unwrap().clone();
    tokio::spawn(async move {
        let _ = writer.lock().await.write_all(format!("{}\n", msg).as_bytes()).await;
    });

    
    siv.call_on_name("input", |view: &mut EditView| {
        view.set_content("");
    });
}


fn create_space_theme() -> Theme {
    let mut theme = Theme::default();
    theme.shadow = true;
    theme.borders = BorderStyle::Simple;

    let mut palette = Palette::default();
    palette[PaletteColor::Background] = Color::Rgb(8, 8, 32);            // Deep space blue-black
    palette[PaletteColor::View] = Color::Rgb(20, 16, 48);                // Slightly lighter, cosmic purple
    palette[PaletteColor::Primary] = Color::Rgb(0, 255, 255);            // Neon cyan for main text
    palette[PaletteColor::TitlePrimary] = Color::Rgb(180, 0, 255);       // Electric purple for titles
    palette[PaletteColor::Secondary] = Color::Rgb(0, 200, 255);          // Blue for secondary elements
    palette[PaletteColor::Highlight] = Color::Rgb(255, 255, 0);          // Bright yellow highlight (stars)
    palette[PaletteColor::HighlightInactive] = Color::Rgb(80, 80, 120);  // Dimmed blue for inactive
    palette[PaletteColor::Shadow] = Color::Rgb(0, 0, 0);                 // Black shadow
    theme.palette = palette;
    theme
}


fn emojify(text: &str) -> String {
    // Replace text with emojis 
    text.replace(":)", "😊")
        .replace(":(", "😢")
        .replace(":D", "😄")
        .replace("<3", "❤️")
        .replace(":/", "😕")
        .replace("XD", "😂")
        .replace("!?", "❓❗")
        .replace("...", "😶")
        .replace(":-)", "😊")
        .replace(":-(", "😢")
        .replace("wtf", "🤬")
        .replace("brb", "🏃‍♂️")
        .replace(";)", "😉")
}



// Function to generate a color based on the username, makes it easier to distinguish username from chat messages 

fn color_for_username(username: &str) -> ColorStyle {
    let colors = [
        Color::Light(BaseColor::Red),
        Color::Light(BaseColor::Green),
        Color::Light(BaseColor::Yellow),
        Color::Light(BaseColor::Blue),
        Color::Light(BaseColor::Magenta),
        Color::Light(BaseColor::Cyan),
    ];
    let idx = username.bytes().fold(0u8, |acc, b| acc.wrapping_add(b)) as usize % colors.len();
    ColorStyle::new(colors[idx], Color::TerminalDefault)
}