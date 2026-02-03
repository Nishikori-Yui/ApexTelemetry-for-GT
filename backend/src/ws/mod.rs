// WebSocket transport layer for telemetry streaming.

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State as AxumState;
use axum::response::IntoResponse;
use futures::StreamExt;
use serde::Serialize;
use tokio::sync::broadcast;
use tracing::{info, warn};

use crate::app::AppState;
use crate::constants::SCHEMA_VERSION;
use crate::model::{Sample, State as TelemetryState};
use crate::utils::{monotonic_ms, next_sequence, now_epoch_ms};

#[derive(Serialize)]
pub struct HandshakeHello {
    pub schema_version: &'static str,
    pub timestamp_ms: u64,
    pub monotonic_ms: u64,
    pub sequence: u64,
    #[serde(rename = "type")]
    pub message_type: &'static str,
    pub server_version: &'static str,
    pub capabilities: Vec<&'static str>,
}

#[derive(Serialize)]
pub struct StateUpdateMessage {
    pub schema_version: &'static str,
    pub timestamp_ms: u64,
    pub monotonic_ms: u64,
    pub sequence: u64,
    #[serde(rename = "type")]
    pub message_type: &'static str,
    pub state: TelemetryState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_timestamp_ms: Option<u64>,
}

#[derive(Serialize)]
pub struct SamplesWindow {
    pub start_ms: u64,
    pub end_ms: u64,
    pub stride_ms: u64,
    pub samples: Vec<Sample>,
}

#[derive(Serialize)]
pub struct SamplesWindowMessage {
    pub schema_version: &'static str,
    pub timestamp_ms: u64,
    pub monotonic_ms: u64,
    pub sequence: u64,
    #[serde(rename = "type")]
    pub message_type: &'static str,
    pub window: SamplesWindow,
    pub decimated: bool,
}

pub async fn ws_handler(
    AxumState(app_state): AxumState<AppState>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, app_state))
}

async fn handle_socket(mut socket: WebSocket, app_state: AppState) {
    info!("ws connected");
    let mut rx = app_state.tx.subscribe();
    let hello = HandshakeHello {
        schema_version: SCHEMA_VERSION,
        timestamp_ms: now_epoch_ms(),
        monotonic_ms: monotonic_ms(app_state.start_instant),
        sequence: next_sequence(app_state.sequence.as_ref()),
        message_type: "handshake_hello",
        server_version: env!("CARGO_PKG_VERSION"),
        capabilities: vec!["state_update", "samples_window"],
    };

    if let Ok(payload) = serde_json::to_string(&hello) {
        if socket.send(Message::Text(payload)).await.is_err() {
            return;
        }
    }

    loop {
        tokio::select! {
            outbound = rx.recv() => {
                match outbound {
                    Ok(payload) => {
                        if socket.send(Message::Text(payload)).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => {
                        continue;
                    }
                    Err(_) => break,
                }
            }
            inbound = socket.next() => {
                match inbound {
                    Some(Ok(Message::Ping(payload))) => {
                        if socket.send(Message::Pong(payload)).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(Message::Close(_))) => break,
                    Some(Ok(_)) => {}
                    Some(Err(err)) => {
                        warn!(?err, "ws error");
                        break;
                    }
                    None => break,
                }
            }
        }
    }
    info!("ws disconnected");
}
