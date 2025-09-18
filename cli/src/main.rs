use clap::{Parser, Subcommand};
use reqwest::Client;

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    /// API base URL
    #[arg(short, long, default_value = "http://127.0.0.1:3000")]
    api: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Send a log message to the server
    Send { message: String },

    /// List logs from the server
    List,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let client = Client::new();

    match cli.command {
        Commands::Send { message } => {
            #[derive(serde::Serialize)]
            struct NewLog<'a> {
                message: &'a str,
            }

            let body = NewLog { message: &message };

            let resp = client
                .post(format!("{}/logs", cli.api))
                .json(&body)
                .send()
                .await?;

            let created: common::LogEntry = resp.json().await?;
            println!("Created: {:?}", created);
        }

        Commands::List => {
            let resp = client.get(format!("{}/logs", cli.api)).send().await?;
            let list: Vec<common::LogEntry> = resp.json().await?;
            for l in list {
                println!("{} | {} {}", l.id, l.timestamp, l.message);
            }
        }
    }

    Ok(())
}
