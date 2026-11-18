//! Makes this machine a tool host: connects to the server with its
//! persistent host id and runs tools for sessions whose active host it is.
//! Used by `co tool-host` and in-process by the desktop app.

use std::path::PathBuf;

use co_client::Client;
use co_protocol::{ClientKind, Hello, HostId};

pub struct Config {
    /// e.g. `ws://localhost:7420/ws`
    pub server_url: String,
    /// Shown to users (defaults to the hostname in `co tool-host`).
    pub name: String,
    /// Overrides the id kept in `~/.co/host-id`.
    pub host_id: Option<String>,
}

/// Connects and serves tool calls until the connection ends.
pub async fn run(config: Config) -> anyhow::Result<()> {
    let host_id = match config.host_id {
        Some(id) => HostId(id),
        None => host_id()?,
    };
    let hello = Hello {
        name: config.name,
        client: ClientKind::ToolHost,
        tools: true,
        host_id: Some(host_id.clone()),
        token: None,
    };
    let (client, mut events) = Client::connect(&config.server_url, hello).await?;
    tracing::info!(%host_id, client_id = %client.hello.client_id, "connected as tool host");
    // TODO: handle `tool_exec` events with `co_tools::execute`.
    while let Some(event) = events.recv().await {
        tracing::debug!(event = %event.event, "event");
    }
    Ok(())
}

/// This machine's persistent host id, created on first use.
pub fn host_id() -> anyhow::Result<HostId> {
    let path = host_id_path()?;
    if let Ok(id) = std::fs::read_to_string(&path) {
        let id = id.trim();
        if !id.is_empty() {
            return Ok(HostId(id.to_string()));
        }
    }
    let id = format!("host-{}", uuid::Uuid::new_v4().simple());
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    // Write to a temporary file and link it into place, so that processes
    // starting together agree on one id.
    let tmp = path.with_extension(format!("tmp-{}", std::process::id()));
    std::fs::write(&tmp, &id)?;
    let linked = std::fs::hard_link(&tmp, &path);
    let _ = std::fs::remove_file(&tmp);
    match linked {
        Ok(()) => Ok(HostId(id)),
        Err(_) => Ok(HostId(std::fs::read_to_string(&path)?.trim().to_string())),
    }
}

fn host_id_path() -> anyhow::Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("no home directory"))?;
    Ok(home.join(".co").join("host-id"))
}
