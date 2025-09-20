use clap::{Parser, Subcommand};
use common::LogEntry;
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
    List {
        #[arg(long)]
        after: Option<String>,

        #[arg(long)]
        contains: Option<String>,

        #[arg(long)]
        limit: Option<String>,

        #[arg(long)]
        offset: Option<String>,
    },

    /// Number of elements in logs
    Count,

    /// Ping the server
    Ping,
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

            let _resp = client
                .post(format!("{}/logs", cli.api))
                .json(&body)
                .send()
                .await?;
        }

        Commands::List {
            after,
            contains,
            limit,
            offset,
        } => {
            let mut url = format!("{}/logs?", cli.api);
            let mut params = vec![];

            if let Some(a) = after {
                params.push(format!("after={a}"));
            }

            if let Some(c) = contains {
                params.push(format!("contains={c}"));
            }

            if let Some(l) = limit {
                params.push(format!("limit={l}"));
            }

            if let Some(o) = offset {
                params.push(format!("offset={o}"));
            }

            url.push_str(&params.join("&"));

            let resp = client.get(url).send().await?;
            let list: Vec<common::LogEntry> = resp.json().await?;
            for l in list {
                println!("[{}] {}", l.timestamp, l.message);
            }
        }

        Commands::Count => {
            let resp = client.get(format!("{}/logs/count", cli.api)).send().await?;
            let count_resp: LogEntry = resp.json().await?;
            println!("{count_resp:?}");
        }

        Commands::Ping => {
            let resp = client.get(format!("{}/ping", cli.api)).send().await?;
            let ping_resp: LogEntry = resp.json().await?;
            println!("{ping_resp:?}");
        }
    }

    Ok(())
}
