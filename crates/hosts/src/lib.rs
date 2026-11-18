//! The registry of connected tool hosts.
//!
//! Sessions refer to hosts by [`HostId`], never by connection, so a session
//! can switch hosts mid-conversation and a host that reconnects is the same
//! host. Still to come here: routing tool calls to a host's connection,
//! tracking in-flight calls, and handing a host id over between connections.

use std::collections::HashMap;
use std::sync::Arc;

use co_protocol::{HostId, HostInfo};
use tokio::sync::RwLock;

/// A connected host.
#[derive(Debug, Clone)]
pub struct Host {
    pub info: HostInfo,
    /// The client connection currently holding this host id.
    pub client_id: String,
}

#[derive(Debug, Clone, Default)]
pub struct Registry {
    hosts: Arc<RwLock<HashMap<HostId, Host>>>,
}

impl Registry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Records `host` as connected, replacing an older connection with the
    /// same id (a newer connection takes the id over).
    pub async fn connect(&self, host: Host) -> Option<Host> {
        self.hosts.write().await.insert(host.info.id.clone(), host)
    }

    /// Forgets the host if `client_id` still holds it.
    pub async fn disconnect(&self, id: &HostId, client_id: &str) {
        let mut hosts = self.hosts.write().await;
        if hosts.get(id).is_some_and(|h| h.client_id == client_id) {
            hosts.remove(id);
        }
    }

    pub async fn get(&self, id: &HostId) -> Option<Host> {
        self.hosts.read().await.get(id).cloned()
    }

    pub async fn list(&self) -> Vec<HostInfo> {
        self.hosts
            .read()
            .await
            .values()
            .map(|h| h.info.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn host(client: &str) -> Host {
        Host {
            info: HostInfo {
                id: HostId("host-1".into()),
                name: "m".into(),
                home: None,
                connected: true,
            },
            client_id: client.into(),
        }
    }

    #[tokio::test]
    async fn newer_connection_takes_over() {
        let r = Registry::new();
        r.connect(host("c1")).await;
        assert!(r.connect(host("c2")).await.is_some());
        // The old connection going away does not drop the host.
        r.disconnect(&HostId("host-1".into()), "c1").await;
        assert_eq!(
            r.get(&HostId("host-1".into())).await.unwrap().client_id,
            "c2"
        );
        r.disconnect(&HostId("host-1".into()), "c2").await;
        assert!(r.list().await.is_empty());
    }
}
