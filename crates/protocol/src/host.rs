//! Tool hosts: machines that run a session's tools.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// A machine's persistent id (kept in `~/.co/host-id`), so a host that
/// reconnects or restarts is the same host and sessions can move back to it.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct HostId(pub String);

impl std::fmt::Display for HostId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// A host as listed to clients.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct HostInfo {
    pub id: HostId,
    pub name: String,
    /// The host's home directory, for `~` in paths meant for it.
    pub home: Option<String>,
    pub connected: bool,
}
