//! The agent loop. One [`Agent`] per session, running in the server; its tool
//! calls go to the session's active host (or a host named per call), so the
//! session can switch hosts at any time.

use co_hosts::Registry;
use co_protocol::HostId;

/// Where a session's tools run, and what it remembers about each host.
#[derive(Debug, Clone, Default)]
pub struct Location {
    pub active_host: Option<HostId>,
    /// Pinned by the user (vs. a default that a tool-capable client may take over).
    pub pinned: bool,
    /// The session's working directory on each host it has used.
    pub cwds: std::collections::HashMap<HostId, String>,
}

pub struct Agent {
    pub session_id: String,
    pub location: Location,
    hosts: Registry,
}

impl Agent {
    pub fn new(session_id: String, hosts: Registry) -> Self {
        Self {
            session_id,
            location: Location::default(),
            hosts,
        }
    }

    /// Moves the session to `host` (mid-session switching): new tool calls
    /// go there, in the cwd the session last had on it.
    pub async fn switch_host(&mut self, host: HostId, pinned: bool) -> anyhow::Result<()> {
        if self.hosts.get(&host).await.is_none() {
            anyhow::bail!("host {host} is not connected");
        }
        self.location.active_host = Some(host);
        self.location.pinned = pinned;
        Ok(())
    }
}
