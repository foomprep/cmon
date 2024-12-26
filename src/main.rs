mod inference;
mod chat;
mod tree;
mod tooler;

use std::fs::OpenOptions;

use chat::ChatUI;
use clap::{Parser, Subcommand};
use crossterm::{event::{self, Event, KeyCode}, terminal};
use env_logger::{Builder, Target};
use inference::{ContentItem, Message, Role};

struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
    }
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    force: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Init,
    Chat,
}

fn setup_logger() -> Result<(), Box<dyn std::error::Error>> {
    // Create or append to log file
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(".prodomme.log")?;

    // Create a custom logger configuration
    let mut builder = Builder::from_default_env();
    builder
        .target(Target::Pipe(Box::new(file)))
        .format_timestamp_secs()
        .init();

    Ok(())
}

async fn run_chat() -> Result<(), Box<dyn std::error::Error>> {
    terminal::enable_raw_mode()?;
    let _guard = TerminalGuard;  
    
    let mut chat = ChatUI::new();
    chat.render()?;

    loop {
        if let Event::Key(key_event) = event::read()? {
            match key_event.code {
                KeyCode::Esc => {
                    chat.cleanup()?;
                    break;
                }
                KeyCode::Enter => {
                    if !chat.input_buffer.is_empty() {
                        if chat.input_buffer == "/exit" {
                            chat.cleanup()?;
                            break;
                        }
                        let user_input = std::mem::take(&mut chat.input_buffer);
                        let new_message = Message {
                            role: Role::User,
                            content: vec![
                                ContentItem::Text { text: user_input } 
                            ]
                        };
                        chat.add_message(new_message).await?;
                    }
                }

                KeyCode::Backspace => {
                    chat.input_buffer.pop();
                }
                KeyCode::Char(c) => {
                    chat.input_buffer.push(c);
                }
                _ => {}
            }
            chat.render()?;
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    setup_logger()?;

    match &cli.command {
        Some(Commands::Init) => {
            println!("Initializing new project");
            // Perform initialization logic here
        }
        Some(Commands::Chat) => {
            if let Err(e) = run_chat().await {
                // Ensure we clean up even on error
                terminal::disable_raw_mode()?;
                return Err(e);
            }
        }
        None => {
            println!("No subcommand provided");
        }
    }

    Ok(())
}

