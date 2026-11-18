//! A connection to the co server: sends `hello`, then multiplexes requests
//! (answered by id) and pushed events.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use co_protocol::{ClientMessage, Event, Hello, HelloReply, Request, Response, ServerMessage};
use futures::{SinkExt, StreamExt};
use tokio::sync::{Mutex, mpsc, oneshot};
use tokio_tungstenite::tungstenite::Message;

type Pending = Arc<Mutex<HashMap<u64, oneshot::Sender<Response>>>>;

pub struct Client {
    pub hello: HelloReply,
    out: mpsc::UnboundedSender<ClientMessage>,
    pending: Pending,
    next_id: AtomicU64,
}

impl Client {
    /// Connects to `url` (e.g. `ws://localhost:7420/ws`) and says `hello`.
    /// Events pushed by the server arrive on the returned receiver.
    pub async fn connect(
        url: &str,
        hello: Hello,
    ) -> anyhow::Result<(Self, mpsc::UnboundedReceiver<Event>)> {
        let (ws, _) = tokio_tungstenite::connect_async(url).await?;
        let (mut sink, mut stream) = ws.split();

        sink.send(Message::text(serde_json::to_string(
            &ClientMessage::Hello(hello),
        )?))
        .await?;
        let reply = loop {
            match stream
                .next()
                .await
                .ok_or_else(|| anyhow::anyhow!("closed before hello_reply"))??
            {
                Message::Text(t) => match serde_json::from_str(&t)? {
                    ServerMessage::HelloReply(r) => break r,
                    other => anyhow::bail!("expected hello_reply, got {other:?}"),
                },
                Message::Close(_) => anyhow::bail!("closed before hello_reply"),
                _ => {}
            }
        };

        let (out, mut out_rx) = mpsc::unbounded_channel::<ClientMessage>();
        tokio::spawn(async move {
            while let Some(msg) = out_rx.recv().await {
                let Ok(text) = serde_json::to_string(&msg) else {
                    continue;
                };
                if sink.send(Message::text(text)).await.is_err() {
                    break;
                }
            }
        });

        let pending: Pending = Arc::default();
        let (events_tx, events) = mpsc::unbounded_channel();
        let pending_in = pending.clone();
        tokio::spawn(async move {
            while let Some(Ok(msg)) = stream.next().await {
                let Message::Text(text) = msg else { continue };
                match serde_json::from_str::<ServerMessage>(&text) {
                    Ok(ServerMessage::Response(r)) => {
                        if let Some(tx) = pending_in.lock().await.remove(&r.id) {
                            let _ = tx.send(r);
                        }
                    }
                    Ok(ServerMessage::Event(e)) => {
                        let _ = events_tx.send(e);
                    }
                    Ok(ServerMessage::HelloReply(_)) => {}
                    Err(e) => tracing::warn!("bad message from server: {e}"),
                }
            }
        });

        Ok((
            Self {
                hello: reply,
                out,
                pending,
                next_id: AtomicU64::new(1),
            },
            events,
        ))
    }

    /// Sends a request and waits for its response.
    pub async fn request(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> anyhow::Result<Response> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let (tx, rx) = oneshot::channel();
        self.pending.lock().await.insert(id, tx);
        self.out.send(ClientMessage::Request(Request {
            id,
            method: method.into(),
            params,
        }))?;
        Ok(rx.await?)
    }
}
