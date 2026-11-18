//! The co wire protocol.
//!
//! Every connection (web, TUI, desktop, tool host) exchanges JSON messages:
//! one per WebSocket text frame. A client sends [`ClientMessage::Hello`]
//! first, then [`Request`]s; the server answers each with a [`Response`]
//! carrying the same id and pushes [`Event`]s for everything asynchronous.
//!
//! These types are the single source of truth: `just codegen` exports them
//! to `packages/protocol/src/generated` for the TypeScript clients.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub mod host;

pub use host::{HostId, HostInfo};

/// Default port of `co serve`.
pub const DEFAULT_PORT: u16 = 7420;

/// Protocol version, sent in [`HelloReply`].
pub const PROTOCOL_VERSION: u32 = 1;

/// What kind of client a connection is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum ClientKind {
    Web,
    Tui,
    Desktop,
    ToolHost,
}

/// The first message of every connection.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Hello {
    /// Human-readable name of the client (e.g. the machine's hostname).
    pub name: String,
    pub client: ClientKind,
    /// Whether this client can run tools, i.e. is a tool host.
    pub tools: bool,
    /// The machine's persistent host id, for tool hosts.
    pub host_id: Option<HostId>,
    /// Session token from a login.
    pub token: Option<String>,
}

/// The server's answer to [`Hello`].
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct HelloReply {
    pub protocol: u32,
    pub client_id: String,
    pub host_id: Option<HostId>,
}

/// A request: `method` names what to do, `params` its arguments.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Request {
    #[ts(type = "number")]
    pub id: u64,
    pub method: String,
    #[ts(type = "unknown")]
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
}

/// The answer to the [`Request`] with the same `id`.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Response {
    #[ts(type = "number")]
    pub id: u64,
    pub ok: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional, type = "unknown")]
    pub result: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub error: Option<ErrorBody>,
}

impl Response {
    pub fn ok(id: u64, result: serde_json::Value) -> Self {
        Self {
            id,
            ok: true,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: u64, code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            id,
            ok: false,
            result: None,
            error: Some(ErrorBody {
                code: code.into(),
                message: message.into(),
            }),
        }
    }
}

/// Something that happened asynchronously (streaming deltas, tool output,
/// state changes).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Event {
    pub event: String,
    #[ts(type = "unknown")]
    pub data: serde_json::Value,
}

/// Client → server.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "type", rename_all = "snake_case")]
#[ts(export)]
pub enum ClientMessage {
    Hello(Hello),
    Request(Request),
}

/// Server → client.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "type", rename_all = "snake_case")]
#[ts(export)]
pub enum ServerMessage {
    HelloReply(HelloReply),
    Response(Response),
    Event(Event),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hello_round_trips() {
        let msg = ClientMessage::Hello(Hello {
            name: "laptop".into(),
            client: ClientKind::ToolHost,
            tools: true,
            host_id: Some(HostId("host-abc".into())),
            token: None,
        });
        let json = serde_json::to_value(&msg).unwrap();
        assert_eq!(json["type"], "hello");
        assert_eq!(json["client"], "tool_host");
        assert_eq!(json["host_id"], "host-abc");
        let back: ClientMessage = serde_json::from_value(json).unwrap();
        assert!(matches!(back, ClientMessage::Hello(h) if h.tools));
    }

    #[test]
    fn response_omits_missing_fields() {
        let json = serde_json::to_value(ServerMessage::Response(Response::ok(
            7,
            serde_json::json!({"x": 1}),
        )))
        .unwrap();
        assert_eq!(
            json,
            serde_json::json!({"type": "response", "id": 7, "ok": true, "result": {"x": 1}})
        );
    }
}
