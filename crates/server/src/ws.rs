//! One WebSocket connection: `hello` first, then requests.

use axum::{
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::Response as HttpResponse,
};
use co_hosts::Host;
use co_protocol::{
    ClientMessage, Hello, HelloReply, HostInfo, PROTOCOL_VERSION, Request, Response, ServerMessage,
};
use futures::StreamExt;

use crate::AppState;

pub async fn upgrade(ws: WebSocketUpgrade, State(state): State<AppState>) -> HttpResponse {
    ws.on_upgrade(move |socket| async move {
        if let Err(e) = connection(socket, state).await {
            tracing::debug!("connection ended: {e:#}");
        }
    })
}

async fn send(ws: &mut WebSocket, msg: &ServerMessage) -> anyhow::Result<()> {
    ws.send(Message::Text(serde_json::to_string(msg)?.into()))
        .await?;
    Ok(())
}

async fn recv(ws: &mut WebSocket) -> anyhow::Result<Option<ClientMessage>> {
    while let Some(msg) = ws.next().await {
        match msg? {
            Message::Text(text) => return Ok(Some(serde_json::from_str(&text)?)),
            Message::Close(_) => return Ok(None),
            _ => {}
        }
    }
    Ok(None)
}

async fn connection(mut ws: WebSocket, state: AppState) -> anyhow::Result<()> {
    let client_id = uuid::Uuid::now_v7().to_string();
    let hello: Hello = match recv(&mut ws).await? {
        Some(ClientMessage::Hello(hello)) => hello,
        Some(_) => anyhow::bail!("expected hello first"),
        None => return Ok(()),
    };
    // TODO: authenticate `hello.token` against the store.
    tracing::info!(%client_id, name = %hello.name, kind = ?hello.client, "client connected");

    let host_id = hello.tools.then(|| hello.host_id.clone()).flatten();
    if let Some(id) = &host_id {
        let info = HostInfo {
            id: id.clone(),
            name: hello.name.clone(),
            home: None,
            connected: true,
        };
        state
            .hosts
            .connect(Host {
                info,
                client_id: client_id.clone(),
            })
            .await;
    }
    let reply = HelloReply {
        protocol: PROTOCOL_VERSION,
        client_id: client_id.clone(),
        host_id: host_id.clone(),
    };
    send(&mut ws, &ServerMessage::HelloReply(reply)).await?;

    let result = async {
        while let Some(msg) = recv(&mut ws).await? {
            match msg {
                ClientMessage::Request(req) => {
                    let resp = handle(&state, req).await;
                    send(&mut ws, &ServerMessage::Response(resp)).await?;
                }
                ClientMessage::Hello(_) => {}
            }
        }
        anyhow::Ok(())
    }
    .await;

    if let Some(id) = &host_id {
        state.hosts.disconnect(id, &client_id).await;
    }
    tracing::info!(%client_id, "client disconnected");
    result
}

async fn handle(state: &AppState, req: Request) -> Response {
    match req.method.as_str() {
        "ping" => Response::ok(req.id, serde_json::json!("pong")),
        "hosts.list" => Response::ok(req.id, serde_json::json!(state.hosts.list().await)),
        other => Response::error(req.id, "unknown_method", format!("unknown method {other}")),
    }
}
