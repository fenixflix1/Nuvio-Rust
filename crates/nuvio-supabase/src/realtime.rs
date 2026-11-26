use crate::{Result, SupabaseConfig};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhoenixFrame {
    pub topic: String,
    pub event: String,
    pub payload: Value,
    #[serde(rename = "ref")]
    pub ref_id: Option<String>,
}

/// Realtime Phoenix WebSocket connection over `tokio-tungstenite`.
pub struct RealtimeConnection {
    outbound_tx: mpsc::Sender<PhoenixFrame>,
    ref_counter: AtomicU64,
}

impl RealtimeConnection {
    pub async fn connect(
        config: &SupabaseConfig,
        _access_token: Option<&str>,
    ) -> Result<(Self, mpsc::Receiver<PhoenixFrame>)> {
        let ws_url = config.realtime_ws_url()?;
        let (ws_stream, _) = connect_async(&ws_url).await?;
        let (mut write, mut read) = ws_stream.split();

        let (out_tx, mut out_rx) = mpsc::channel::<PhoenixFrame>(64);
        let (in_tx, in_rx) = mpsc::channel::<PhoenixFrame>(128);

        tokio::spawn(async move {
            while let Some(frame) = out_rx.recv().await {
                if let Ok(text) = serde_json::to_string(&frame)
                    && write.send(Message::Text(text.into())).await.is_err()
                {
                    break;
                }
            }
        });

        tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                if let Ok(Message::Text(txt)) = msg
                    && let Ok(frame) = serde_json::from_str::<PhoenixFrame>(&txt)
                    && in_tx.send(frame).await.is_err()
                {
                    break;
                }
            }
        });

        let conn = Self {
            outbound_tx: out_tx,
            ref_counter: AtomicU64::new(1),
        };

        Ok((conn, in_rx))
    }

    fn next_ref(&self) -> String {
        self.ref_counter.fetch_add(1, Ordering::Relaxed).to_string()
    }

    pub async fn join_channel(&self, topic: &str, payload: Value) -> Result<()> {
        let frame = PhoenixFrame {
            topic: topic.to_string(),
            event: "phx_join".to_string(),
            payload,
            ref_id: Some(self.next_ref()),
        };
        let _ = self.outbound_tx.send(frame).await;
        Ok(())
    }

    pub async fn leave_channel(&self, topic: &str) -> Result<()> {
        let frame = PhoenixFrame {
            topic: topic.to_string(),
            event: "phx_leave".to_string(),
            payload: serde_json::json!({}),
            ref_id: Some(self.next_ref()),
        };
        let _ = self.outbound_tx.send(frame).await;
        Ok(())
    }

    pub async fn broadcast(&self, topic: &str, event: &str, payload: Value) -> Result<()> {
        let frame = PhoenixFrame {
            topic: topic.to_string(),
            event: "broadcast".to_string(),
            payload: serde_json::json!({
                "type": "broadcast",
                "event": event,
                "payload": payload,
            }),
            ref_id: Some(self.next_ref()),
        };
        let _ = self.outbound_tx.send(frame).await;
        Ok(())
    }

    pub async fn track_presence(&self, topic: &str, payload: Value) -> Result<()> {
        let frame = PhoenixFrame {
            topic: topic.to_string(),
            event: "presence".to_string(),
            payload: serde_json::json!({
                "type": "presence",
                "event": "track",
                "payload": payload,
            }),
            ref_id: Some(self.next_ref()),
        };
        let _ = self.outbound_tx.send(frame).await;
        Ok(())
    }

    pub async fn heartbeat(&self) -> Result<()> {
        let frame = PhoenixFrame {
            topic: "phoenix".to_string(),
            event: "heartbeat".to_string(),
            payload: serde_json::json!({}),
            ref_id: Some(self.next_ref()),
        };
        let _ = self.outbound_tx.send(frame).await;
        Ok(())
    }

    pub async fn send_access_token(&self, topic: &str, token: &str) -> Result<()> {
        let frame = PhoenixFrame {
            topic: topic.to_string(),
            event: "access_token".to_string(),
            payload: serde_json::json!({ "access_token": token }),
            ref_id: Some(self.next_ref()),
        };
        let _ = self.outbound_tx.send(frame).await;
        Ok(())
    }
}
