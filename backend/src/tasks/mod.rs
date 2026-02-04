// Background tasks for websocket updates, samples, and heartbeat.

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{broadcast, watch, RwLock};
use tokio::time::{self, Instant};
use tracing::{info, warn};

use crate::app::{DetectStatus, DetectStore, HeartbeatMode, SessionState, TelemetryStore, UdpConfig};
use crate::constants::{
    HEARTBEAT_BYTE, HEARTBEAT_INTERVAL_SECS, HEARTBEAT_PORT, SCHEMA_VERSION, STATE_INTERVAL_MS,
    WINDOW_DURATION_MS, WINDOW_INTERVAL_MS, WINDOW_STRIDE_MS,
};
use crate::net::{
    bind_heartbeat_socket, fallback_local_ip, resolve_broadcast_bind_ip, resolve_local_ip_for_target,
};
use crate::utils::{monotonic_ms, next_sequence, now_epoch_ms};
use crate::ws::{SamplesWindow, SamplesWindowMessage, StateUpdateMessage};

pub async fn state_update_task(
    store: Arc<RwLock<TelemetryStore>>,
    tx: broadcast::Sender<String>,
    sequence: Arc<AtomicU64>,
    start: Instant,
) {
    let mut interval = time::interval(Duration::from_millis(STATE_INTERVAL_MS));
    loop {
        interval.tick().await;
        let (state, source_timestamp_ms) = {
            let store = store.read().await;
            (store.session.state.clone(), store.last_source_timestamp_ms)
        };

        if state.is_empty() {
            continue;
        }

        let message = StateUpdateMessage {
            schema_version: SCHEMA_VERSION,
            timestamp_ms: now_epoch_ms(),
            monotonic_ms: monotonic_ms(start),
            sequence: next_sequence(sequence.as_ref()),
            message_type: "state_update",
            state,
            source_timestamp_ms,
        };

        if let Ok(payload) = serde_json::to_string(&message) {
            let _ = tx.send(payload);
        }
    }
}

pub async fn samples_window_task(
    store: Arc<RwLock<TelemetryStore>>,
    tx: broadcast::Sender<String>,
    sequence: Arc<AtomicU64>,
    start: Instant,
) {
    let mut interval = time::interval(Duration::from_millis(WINDOW_INTERVAL_MS));
    loop {
        interval.tick().await;
        let now_ms = monotonic_ms(start);
        let start_ms = now_ms.saturating_sub(WINDOW_DURATION_MS);
        let samples = {
            let store = store.read().await;
            if store.session.session_state != SessionState::InRace {
                continue;
            }
            if store.samples.len() == 0 {
                continue;
            }
            store.samples.to_vec_ordered()
        };

        let mut window_samples = Vec::new();
        let mut last_t = None;
        for sample in samples {
            if sample.t_ms < start_ms || sample.t_ms > now_ms {
                continue;
            }
            let emit = match last_t {
                Some(prev) => sample.t_ms.saturating_sub(prev) >= WINDOW_STRIDE_MS,
                None => true,
            };
            if emit {
                last_t = Some(sample.t_ms);
                window_samples.push(sample);
            }
        }

        if window_samples.is_empty() {
            continue;
        }

        let window = SamplesWindow {
            start_ms,
            end_ms: now_ms,
            stride_ms: WINDOW_STRIDE_MS,
            samples: window_samples,
        };

        let message = SamplesWindowMessage {
            schema_version: SCHEMA_VERSION,
            timestamp_ms: now_epoch_ms(),
            monotonic_ms: monotonic_ms(start),
            sequence: next_sequence(sequence.as_ref()),
            message_type: "samples_window",
            window,
            decimated: true,
        };

        if let Ok(payload) = serde_json::to_string(&message) {
            let _ = tx.send(payload);
        }
    }
}

pub async fn heartbeat_task(
    mut config_rx: watch::Receiver<UdpConfig>,
    detect_store: Arc<RwLock<DetectStore>>,
    store: Arc<RwLock<TelemetryStore>>,
    start: Instant,
) -> std::io::Result<()> {
    let mut socket = bind_heartbeat_socket(IpAddr::V4(Ipv4Addr::UNSPECIFIED)).await?;
    let mut current_bind_ip = IpAddr::V4(Ipv4Addr::UNSPECIFIED);
    info!(local_addr = %socket.local_addr()?, "heartbeat task started");

    let mut current_mode = HeartbeatMode::Stop;
    let mut config = config_rx.borrow().clone();
    let mut last_ps5_ip = config.ps5_ip;
    let mut last_warn_ms: Option<u64> = None;
    let mut interval = time::interval(Duration::from_secs(HEARTBEAT_INTERVAL_SECS));

    loop {
        tokio::select! {
            _ = interval.tick() => {
                apply_heartbeat_mode(
                    &mut current_mode,
                    &mut last_ps5_ip,
                    &mut last_warn_ms,
                    &mut current_bind_ip,
                    &mut socket,
                    &config,
                    &detect_store,
                    &store,
                    start,
                    true,
                )
                .await?;
            }
            changed = config_rx.changed() => {
                if changed.is_err() {
                    break;
                }
                config = config_rx.borrow().clone();
                apply_heartbeat_mode(
                    &mut current_mode,
                    &mut last_ps5_ip,
                    &mut last_warn_ms,
                    &mut current_bind_ip,
                    &mut socket,
                    &config,
                    &detect_store,
                    &store,
                    start,
                    false,
                )
                .await?;
            }
        }
    }

    Ok(())
}

async fn apply_heartbeat_mode(
    current_mode: &mut HeartbeatMode,
    last_ps5_ip: &mut Option<IpAddr>,
    last_warn_ms: &mut Option<u64>,
    current_bind_ip: &mut IpAddr,
    heartbeat_socket: &mut tokio::net::UdpSocket,
    config: &UdpConfig,
    detect_store: &Arc<RwLock<DetectStore>>,
    store: &Arc<RwLock<TelemetryStore>>,
    start: Instant,
    send_now: bool,
) -> std::io::Result<()> {
    let (pending_detect, last_event) = {
        let store = detect_store.read().await;
        let pending = store
            .active_id
            .and_then(|active_id| store.sessions.get(&active_id))
            .map(|session| matches!(session.status, DetectStatus::Pending))
            .unwrap_or(false);
        (pending, store.last_event.clone())
    };

    let next_mode = if let Some(ip) = config.ps5_ip {
        HeartbeatMode::Unicast(ip)
    } else if pending_detect {
        HeartbeatMode::Broadcast
    } else {
        HeartbeatMode::Stop
    };

    let target_bind_ip = match &next_mode {
        HeartbeatMode::Stop => None,
        HeartbeatMode::Unicast(ip) => resolve_local_ip_for_target(*ip).ok().or_else(|| {
            warn!(%ip, "failed to resolve local ip for unicast heartbeat");
            fallback_local_ip(config)
        }),
        HeartbeatMode::Broadcast => resolve_broadcast_bind_ip(config, pending_detect),
    };

    if let Some(bind_ip) = target_bind_ip {
        if bind_ip != *current_bind_ip {
            let new_socket = bind_heartbeat_socket(bind_ip).await?;
            *heartbeat_socket = new_socket;
            *current_bind_ip = bind_ip;
            info!(local_addr = %heartbeat_socket.local_addr()?, "heartbeat bind updated");
        }
    } else if !matches!(next_mode, HeartbeatMode::Stop) {
        warn!("heartbeat bind ip unavailable; stopping heartbeat");
        *current_mode = HeartbeatMode::Stop;
        return Ok(());
    }

    if &next_mode != current_mode {
        match (&next_mode, last_event.as_ref()) {
            (HeartbeatMode::Broadcast, Some(event)) if matches!(event.status, DetectStatus::Pending) => {
                info!("detect pending -> heartbeat broadcast");
            }
            (HeartbeatMode::Stop, Some(event))
                if matches!(event.status, DetectStatus::Timeout | DetectStatus::Cancelled | DetectStatus::Error)
                    && config.ps5_ip.is_none() =>
            {
                info!("detect terminal without ps5_ip -> heartbeat stop");
            }
            (HeartbeatMode::Unicast(ip), Some(event)) if matches!(event.status, DetectStatus::Found) => {
                info!(%ip, "detect found -> heartbeat unicast");
            }
            (HeartbeatMode::Unicast(ip), _) if last_ps5_ip.is_none() && config.ps5_ip.is_some() => {
                info!(%ip, "manual ps5_ip set -> heartbeat unicast");
            }
            (HeartbeatMode::Stop, _) => {
                info!("heartbeat stopped");
            }
            (HeartbeatMode::Broadcast, _) => {
                info!("heartbeat mode: broadcast");
            }
            (HeartbeatMode::Unicast(ip), _) => {
                info!(%ip, "heartbeat mode: unicast");
            }
        }

        match &next_mode {
            HeartbeatMode::Stop => {
                let _ = heartbeat_socket.set_broadcast(false);
            }
            HeartbeatMode::Broadcast => {
                heartbeat_socket.set_broadcast(true)?;
            }
            HeartbeatMode::Unicast(_) => {
                let _ = heartbeat_socket.set_broadcast(false);
            }
        }
        *current_mode = next_mode;
    }

    if send_now {
        let target = match current_mode {
            HeartbeatMode::Stop => return Ok(()),
            HeartbeatMode::Broadcast => SocketAddr::new(IpAddr::V4(Ipv4Addr::BROADCAST), HEARTBEAT_PORT),
            HeartbeatMode::Unicast(ip) => SocketAddr::new(*ip, HEARTBEAT_PORT),
        };
        info!(local_addr = %heartbeat_socket.local_addr()?, target = %target, "heartbeat send");
        let _ = heartbeat_socket.send_to(&[HEARTBEAT_BYTE], target).await;

        if let HeartbeatMode::Unicast(_) = current_mode {
            let now_ms = monotonic_ms(start);
            let last_ms = store.read().await.last_telemetry_ms;
            if let Some(last_ms) = last_ms {
                if now_ms.saturating_sub(last_ms) >= 5_000 {
                    let should_warn = match last_warn_ms {
                        Some(prev) => now_ms.saturating_sub(*prev) >= 5_000,
                        None => true,
                    };
                    if should_warn {
                        warn!(age_ms = now_ms.saturating_sub(last_ms), "telemetry stale while heartbeat unicast");
                        *last_warn_ms = Some(now_ms);
                    }
                }
            }
        }
    }

    *last_ps5_ip = config.ps5_ip;
    Ok(())
}
