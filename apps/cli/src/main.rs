use std::net::SocketAddr;

use clap::{Parser, Subcommand};

/// co: a coding agent harness with a server, tool hosts and many frontends.
#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    /// The server to connect to (TUI and tool host).
    #[arg(long, global = true, env = "CO_SERVER_URL", default_value_t = default_server_url())]
    server: String,
}

#[derive(Subcommand)]
enum Command {
    /// Run the server.
    Serve {
        #[arg(long, env = "CO_LISTEN", default_value_t = SocketAddr::from(([127, 0, 0, 1], co_protocol::DEFAULT_PORT)))]
        listen: SocketAddr,
        #[arg(long, env = "DATABASE_URL")]
        database_url: String,
    },
    /// Make this machine a tool host of the server.
    ToolHost {
        /// Name shown to users (default: the hostname).
        #[arg(long)]
        name: Option<String>,
        /// Use this host id instead of the one in ~/.co/host-id.
        #[arg(long)]
        host_id: Option<String>,
    },
    /// Open the terminal UI (the default).
    Tui,
}

fn default_server_url() -> String {
    format!("ws://localhost:{}/ws", co_protocol::DEFAULT_PORT)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let command = cli.command.unwrap_or(Command::Tui);
    // The TUI owns the terminal: only log to stderr for the other commands.
    if !matches!(command, Command::Tui) {
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "info,co=debug".into()),
            )
            .init();
    }
    match command {
        Command::Serve {
            listen,
            database_url,
        } => {
            co_server::serve(co_server::Config {
                listen,
                database_url,
            })
            .await
        }
        Command::ToolHost { name, host_id } => {
            let name = name.unwrap_or_else(hostname);
            co_tool_host::run(co_tool_host::Config {
                server_url: cli.server,
                name,
                host_id,
            })
            .await
        }
        Command::Tui => {
            co_tui::run(co_tui::Config {
                server_url: cli.server,
            })
            .await
        }
    }
}

fn hostname() -> String {
    std::env::var("HOSTNAME")
        .ok()
        .or_else(|| {
            std::fs::read_to_string("/etc/hostname")
                .ok()
                .map(|s| s.trim().to_string())
        })
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".into())
}
