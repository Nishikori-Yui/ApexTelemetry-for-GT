// Minimal telemetry pipeline server for gt7-laplab.

use std::collections::HashMap;
use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State as AxumState;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, mpsc, watch, RwLock};
use tokio::time::{self, Instant};
use tracing::{info, warn};

use if_addrs::get_if_addrs;

use gt7_laplab_server::buffers::RingBuffer;
use gt7_laplab_server::crypto;
use gt7_laplab_server::model::{Sample, State as TelemetryState};
use gt7_laplab_server::parser;

const SCHEMA_VERSION: &str = "1.0";
const STATE_INTERVAL_MS: u64 = 50;
const WINDOW_INTERVAL_MS: u64 = 250;
const WINDOW_DURATION_MS: u64 = 5_000;
const WINDOW_STRIDE_MS: u64 = 50;
const SAMPLE_BUFFER_CAP: usize = 600;
const HEARTBEAT_PORT: u16 = 33739;
const HEARTBEAT_INTERVAL_SECS: u64 = 1;
const HEARTBEAT_BYTE: u8 = 0x41;

#[derive(Clone)]
struct AppState {
    tx: broadcast::Sender<String>,
    sequence: Arc<AtomicU64>,
    start_instant: Instant,
    udp_config_tx: watch::Sender<UdpConfig>,
    detect_tx: mpsc::Sender<DetectCommand>,
    detect_store: Arc<RwLock<DetectStore>>,
    detect_sequence: Arc<AtomicU64>,
}

struct TelemetryStore {
    state: TelemetryState,
    samples: RingBuffer<Sample>,
    last_packet_id: Option<i32>,
    last_source_timestamp_ms: Option<u64>,
    last_telemetry_ms: Option<u64>,
    session_state: SessionState,
    session_index: u64,
    last_current_lap: Option<i16>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct UdpConfig {
    bind_addr: IpAddr,
    ps5_ip: Option<IpAddr>,
}

impl Default for UdpConfig {
    fn default() -> Self {
        Self {
            bind_addr: IpAddr::V4(Ipv4Addr::LOCALHOST),
            ps5_ip: None,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum DetectStatus {
    Pending,
    Found,
    Timeout,
    Error,
    Cancelled,
}

#[derive(Clone, Debug)]
struct DetectSession {
    id: u64,
    status: DetectStatus,
    ps5_ip: Option<IpAddr>,
    timeout_ms: u64,
}

#[derive(Clone, Debug)]
struct DetectEvent {
    id: u64,
    status: DetectStatus,
}

#[derive(Default)]
struct DetectStore {
    sessions: HashMap<u64, DetectSession>,
    active_id: Option<u64>,
    last_event: Option<DetectEvent>,
}

#[derive(Serialize)]
struct DetectStartResponse {
    id: u64,
    status: DetectStatus,
    timeout_ms: u64,
}

#[derive(Serialize)]
struct DetectStatusResponse {
    id: u64,
    status: DetectStatus,
    ps5_ip: Option<IpAddr>,
}

struct DetectCommand {
    id: u64,
    timeout_ms: u64,
}

struct DetectState {
    id: u64,
    deadline: Instant,
}

#[derive(Clone, Debug, PartialEq)]
enum HeartbeatMode {
    Stop,
    Broadcast,
    Unicast(IpAddr),
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum SessionState {
    NotInRace,
    InRace,
    Paused,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

#[derive(Serialize)]
struct HandshakeHello {
    schema_version: &'static str,
    timestamp_ms: u64,
    monotonic_ms: u64,
    sequence: u64,
    #[serde(rename = "type")]
    message_type: &'static str,
    server_version: &'static str,
    capabilities: Vec<&'static str>,
}

#[derive(Serialize)]
struct StateUpdateMessage {
    schema_version: &'static str,
    timestamp_ms: u64,
    monotonic_ms: u64,
    sequence: u64,
    #[serde(rename = "type")]
    message_type: &'static str,
    state: TelemetryState,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_timestamp_ms: Option<u64>,
}

#[derive(Serialize)]
struct SamplesWindow {
    start_ms: u64,
    end_ms: u64,
    stride_ms: u64,
    samples: Vec<Sample>,
}

#[derive(Serialize)]
struct SamplesWindowMessage {
    schema_version: &'static str,
    timestamp_ms: u64,
    monotonic_ms: u64,
    sequence: u64,
    #[serde(rename = "type")]
    message_type: &'static str,
    window: SamplesWindow,
    decimated: bool,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let bind = env::var("HTTP_BIND").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("HTTP_PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(10086);
    let addr: SocketAddr = format!("{}:{}", bind, port)
        .parse()
        .expect("invalid HTTP_BIND or HTTP_PORT");

    let udp_bind = env::var("GT7_UDP_BIND").unwrap_or_else(|_| "127.0.0.1".to_string());
    let udp_port = env::var("GT7_UDP_PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(33740);
    let udp_bind_addr = udp_bind
        .parse::<IpAddr>()
        .unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST));

    let store = Arc::new(RwLock::new(TelemetryStore {
        state: TelemetryState::default(),
        samples: RingBuffer::new(SAMPLE_BUFFER_CAP),
        last_packet_id: None,
        last_source_timestamp_ms: None,
        last_telemetry_ms: None,
        session_state: SessionState::NotInRace,
        session_index: 0,
        last_current_lap: None,
    }));

    let (tx, _) = broadcast::channel::<String>(256);
    let (udp_config_tx, udp_config_rx) = watch::channel(UdpConfig {
        bind_addr: udp_bind_addr,
        ps5_ip: None,
    });
    let (detect_tx, detect_rx) = mpsc::channel::<DetectCommand>(8);
    let detect_store = Arc::new(RwLock::new(DetectStore::default()));
    let detect_sequence = Arc::new(AtomicU64::new(0));
    let sequence = Arc::new(AtomicU64::new(0));
    let start_instant = Instant::now();

    let udp_store = store.clone();
    let udp_start = start_instant;
    let udp_detect_store = detect_store.clone();
    let udp_config_tx_udp = udp_config_tx.clone();
    tokio::spawn(async move {
        if let Err(err) = udp_loop(
            udp_port,
            udp_config_rx,
            udp_config_tx_udp,
            detect_rx,
            udp_detect_store,
            udp_store,
            udp_start,
        )
        .await
        {
            warn!(?err, "udp loop exited");
        }
    });

    let state_store = store.clone();
    let state_tx = tx.clone();
    let state_seq = sequence.clone();
    let state_start = start_instant;
    tokio::spawn(async move {
        state_update_task(state_store, state_tx, state_seq, state_start).await;
    });

    let samples_store = store.clone();
    let samples_tx = tx.clone();
    let samples_seq = sequence.clone();
    let samples_start = start_instant;
    tokio::spawn(async move {
        samples_window_task(samples_store, samples_tx, samples_seq, samples_start).await;
    });

    let heartbeat_config_rx = udp_config_tx.subscribe();
    let heartbeat_detect_store = detect_store.clone();
    let heartbeat_store = store.clone();
    let heartbeat_start = start_instant;
    tokio::spawn(async move {
        if let Err(err) = heartbeat_task(
            heartbeat_config_rx,
            heartbeat_detect_store,
            heartbeat_store,
            heartbeat_start,
        )
        .await
        {
            warn!(?err, "heartbeat task exited");
        }
    });

    let app = Router::new()
        .route("/health", get(health))
        .route("/config/udp", get(get_udp_config).post(set_udp_config))
        .route(
            "/config/udp/auto-detect",
            axum::routing::post(start_auto_detect),
        )
        .route(
            "/config/udp/auto-detect/cancel",
            axum::routing::post(cancel_auto_detect),
        )
        .route(
            "/config/udp/auto-detect/:id",
            get(get_auto_detect_status),
        )
        .route("/ws", get(ws_handler))
        .with_state(AppState {
            tx,
            sequence,
            start_instant,
            udp_config_tx,
            detect_tx,
            detect_store,
            detect_sequence,
        });

    info!(%addr, "starting server");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("server failed");
}

async fn health() -> impl IntoResponse {
    Json(HealthResponse { status: "ok" })
}

async fn ws_handler(
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

async fn get_udp_config(AxumState(app_state): AxumState<AppState>) -> impl IntoResponse {
    let config = app_state.udp_config_tx.borrow().clone();
    Json(config)
}

async fn set_udp_config(
    AxumState(app_state): AxumState<AppState>,
    Json(payload): Json<UdpConfig>,
) -> impl IntoResponse {
    if payload.ps5_ip.is_some() {
        let mut store = app_state.detect_store.write().await;
        if let Some(active_id) = store.active_id {
            if let Some(session) = store.sessions.get_mut(&active_id) {
                session.status = DetectStatus::Cancelled;
                session.ps5_ip = None;
                store.last_event = Some(DetectEvent {
                    id: active_id,
                    status: DetectStatus::Cancelled,
                });
                info!(id = active_id, "manual ps5_ip override cancelled auto-detect");
            }
            store.active_id = None;
        }
    }
    let _ = app_state.udp_config_tx.send(payload.clone());
    Json(payload)
}

async fn start_auto_detect(AxumState(app_state): AxumState<AppState>) -> impl IntoResponse {
    let mut store = app_state.detect_store.write().await;
    if let Some(active_id) = store.active_id {
        if let Some(session) = store.sessions.get(&active_id) {
            info!(id = active_id, "auto-detect already active");
            return Json(DetectStartResponse {
                id: session.id,
                status: session.status.clone(),
                timeout_ms: session.timeout_ms,
            });
        }
    }

    let id = next_sequence(app_state.detect_sequence.as_ref());
    let timeout_ms = 10_000;
    let session = DetectSession {
        id,
        status: DetectStatus::Pending,
        ps5_ip: None,
        timeout_ms,
    };
    store.sessions.insert(id, session.clone());
    store.active_id = Some(id);
    store.last_event = Some(DetectEvent {
        id,
        status: DetectStatus::Pending,
    });
    drop(store);

    info!(id, "auto-detect session started");
    let _ = app_state
        .detect_tx
        .send(DetectCommand { id, timeout_ms })
        .await;

    Json(DetectStartResponse {
        id,
        status: DetectStatus::Pending,
        timeout_ms,
    })
}

async fn cancel_auto_detect(AxumState(app_state): AxumState<AppState>) -> impl IntoResponse {
    let mut store = app_state.detect_store.write().await;
    if let Some(active_id) = store.active_id {
        if let Some(session) = store.sessions.get_mut(&active_id) {
            session.status = DetectStatus::Cancelled;
            session.ps5_ip = None;
            store.last_event = Some(DetectEvent {
                id: active_id,
                status: DetectStatus::Cancelled,
            });
        }
        store.active_id = None;
        info!(id = active_id, "auto-detect session cancelled");
        return Json(serde_json::json!({
            "status": "cancelled",
            "id": active_id,
        }));
    }

    info!("auto-detect cancel requested with no active session");
    Json(serde_json::json!({
        "status": "no_active_session",
        "id": null,
    }))
}

async fn get_auto_detect_status(
    AxumState(app_state): AxumState<AppState>,
    axum::extract::Path(id): axum::extract::Path<u64>,
) -> impl IntoResponse {
    let store = app_state.detect_store.read().await;
    if let Some(session) = store.sessions.get(&id) {
        return Json(DetectStatusResponse {
            id: session.id,
            status: session.status.clone(),
            ps5_ip: session.ps5_ip,
        });
    }

    Json(DetectStatusResponse {
        id,
        status: DetectStatus::Error,
        ps5_ip: None,
    })
}

async fn udp_loop(
    udp_port: u16,
    mut config_rx: watch::Receiver<UdpConfig>,
    config_tx: watch::Sender<UdpConfig>,
    mut detect_rx: mpsc::Receiver<DetectCommand>,
    detect_store: Arc<RwLock<DetectStore>>,
    store: Arc<RwLock<TelemetryStore>>,
    start: Instant,
) -> std::io::Result<()> {
    let mut config = config_rx.borrow().clone();
    let mut socket = bind_udp_socket(config.bind_addr, udp_port).await?;
    let mut active_bind = config.bind_addr;
    let mut detect_state: Option<DetectState> = None;
    let mut detect_tick = time::interval(Duration::from_millis(200));
    let mut buf = [0u8; 4096];
    let mut last_inspect_log_ms: u64 = 0;

    loop {
        tokio::select! {
            _ = detect_tick.tick() => {
                if let Some(state) = detect_state.as_ref() {
                    {
                        let store_lock = detect_store.read().await;
                        if store_lock.active_id != Some(state.id) {
                            drop(store_lock);
                            info!(id = state.id, "auto-detect cancelled");
                            detect_state = None;
                            if active_bind != config.bind_addr {
                                match rebind_udp_socket(socket, config.bind_addr, active_bind, udp_port)
                                    .await
                                {
                                    Ok((new_socket, bound_addr, _)) => {
                                        socket = new_socket;
                                        active_bind = bound_addr;
                                    }
                                    Err(err) => {
                                        warn!(?err, "failed to restore udp bind after cancel");
                                        break;
                                    }
                                }
                            }
                            continue;
                        }
                    }
                    if Instant::now() >= state.deadline {
                        let mut store_lock = detect_store.write().await;
                        if let Some(session) = store_lock.sessions.get_mut(&state.id) {
                            session.status = DetectStatus::Timeout;
                            session.ps5_ip = None;
                            store_lock.last_event = Some(DetectEvent {
                                id: state.id,
                                status: DetectStatus::Timeout,
                            });
                        }
                        store_lock.active_id = None;
                        info!(id = state.id, "auto-detect timed out");
                        detect_state = None;
                        drop(store_lock);

                        if active_bind != config.bind_addr {
                            match rebind_udp_socket(socket, config.bind_addr, active_bind, udp_port)
                                .await
                            {
                                Ok((new_socket, bound_addr, _)) => {
                                    socket = new_socket;
                                    active_bind = bound_addr;
                                }
                                Err(err) => {
                                    warn!(?err, "failed to restore udp bind after timeout");
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            cmd = detect_rx.recv() => {
                if let Some(cmd) = cmd {
                    let deadline = Instant::now() + Duration::from_millis(cmd.timeout_ms);
                    if active_bind != IpAddr::V4(Ipv4Addr::UNSPECIFIED) {
                        match rebind_udp_socket(
                            socket,
                            IpAddr::V4(Ipv4Addr::UNSPECIFIED),
                            active_bind,
                            udp_port,
                        )
                        .await
                        {
                            Ok((new_socket, bound_addr, used_fallback)) => {
                                socket = new_socket;
                                active_bind = bound_addr;
                                if used_fallback {
                                    warn!("failed to bind for auto-detect");
                                    let mut store_lock = detect_store.write().await;
                                    if let Some(session) = store_lock.sessions.get_mut(&cmd.id) {
                                        session.status = DetectStatus::Error;
                                        session.ps5_ip = None;
                                        store_lock.last_event = Some(DetectEvent {
                                            id: cmd.id,
                                            status: DetectStatus::Error,
                                        });
                                    }
                                    store_lock.active_id = None;
                                    continue;
                                }
                            }
                            Err(err) => {
                                warn!(?err, "failed to bind for auto-detect");
                                let mut store_lock = detect_store.write().await;
                                if let Some(session) = store_lock.sessions.get_mut(&cmd.id) {
                                    session.status = DetectStatus::Error;
                                    session.ps5_ip = None;
                                    store_lock.last_event = Some(DetectEvent {
                                        id: cmd.id,
                                        status: DetectStatus::Error,
                                    });
                                }
                                store_lock.active_id = None;
                                break;
                            }
                        }
                    }
                    detect_state = Some(DetectState { id: cmd.id, deadline });
                }
            }
            recv = socket.recv_from(&mut buf) => {
                let (len, source) = recv?;
                let payload = match crypto::decrypt_packet(&buf[..len]) {
                    Some(payload) => payload,
                    None => continue,
                };
                let frame = match parser::parse_telemetry(&payload) {
                    Some(frame) => frame,
                    None => continue,
                };
                let flags_byte = payload.get(0x8E).copied();
                let in_race_bit = flags_byte.map(|value| (value & 0b0000_0001) != 0);
                let is_paused_bit = flags_byte.map(|value| (value & 0b0000_0010) != 0);
                let now_ms = monotonic_ms(start);

                if let Some(state) = detect_state.as_ref() {
                    let found_ip = source.ip();
                    let mut store_lock = detect_store.write().await;
                    let mut should_apply = false;
                    if let Some(session) = store_lock.sessions.get_mut(&state.id) {
                        if matches!(session.status, DetectStatus::Pending) {
                            session.status = DetectStatus::Found;
                            session.ps5_ip = Some(found_ip);
                            store_lock.last_event = Some(DetectEvent {
                                id: state.id,
                                status: DetectStatus::Found,
                            });
                            should_apply = true;
                        }
                    }
                    store_lock.active_id = None;
                    drop(store_lock);

                    if should_apply {
                        let mut next_config = config.clone();
                        next_config.ps5_ip = Some(found_ip);
                        if next_config.bind_addr.is_loopback() {
                            next_config.bind_addr = IpAddr::V4(Ipv4Addr::UNSPECIFIED);
                            info!("auto-switched bind_addr to 0.0.0.0 due to loopback");
                        }
                        let _ = config_tx.send(next_config.clone());
                        config = next_config;
                        info!(id = state.id, %found_ip, "auto-detect found: set ps5_ip");
                    } else {
                        info!(id = state.id, %found_ip, "auto-detect result dropped");
                    }
                    detect_state = None;

                    if active_bind != config.bind_addr {
                        match rebind_udp_socket(socket, config.bind_addr, active_bind, udp_port)
                            .await
                        {
                            Ok((new_socket, bound_addr, _)) => {
                                socket = new_socket;
                                active_bind = bound_addr;
                            }
                            Err(err) => {
                                warn!(?err, "failed to restore udp bind after detect");
                                break;
                            }
                        }
                    }
                } else if let Some(ps5_ip) = config.ps5_ip {
                    if source.ip() != ps5_ip {
                        continue;
                    }
                }

                let mut store = store.write().await;
                if now_ms.saturating_sub(last_inspect_log_ms) >= 1000 {
                    last_inspect_log_ms = now_ms;
                    info!(
                        payload_len = payload.len(),
                        payload_base_offset = 0,
                        flags_byte = ?flags_byte.map(|value| format!("0x{value:02X}")),
                        in_race_bit,
                        is_paused_bit,
                        packet_id = frame.packet_id,
                        current_lap = frame.current_lap,
                        time_on_track_ms = frame.time_on_track_ms,
                        speed_kph = frame.speed_kph,
                        rpm = frame.rpm,
                        session_state = ?store.session_state,
                        session_index = store.session_index,
                        "telemetry inspect"
                    );
                }
                store.last_telemetry_ms = Some(monotonic_ms(start));

                let previous_state = store.session_state;
                let mut next_state = previous_state;
                if let (Some(in_race), Some(is_paused)) = (frame.in_race, frame.is_paused) {
                    next_state = if !in_race {
                        SessionState::NotInRace
                    } else if is_paused {
                        SessionState::Paused
                    } else {
                        SessionState::InRace
                    };
                }

                if next_state != previous_state {
                    if next_state == SessionState::InRace
                        && previous_state == SessionState::NotInRace
                    {
                        store.session_index += 1;
                        store.samples.clear();
                        store.last_packet_id = None;
                        store.last_current_lap = None;
                    }
                    store.session_state = next_state;
                    info!(
                        from = ?previous_state,
                        to = ?next_state,
                        session_index = store.session_index,
                        "session transition"
                    );
                }

                store.state.update_from(&frame);
                if frame.current_lap.is_some() {
                    store.last_current_lap = frame.current_lap;
                }

                if store.session_state == SessionState::InRace {
                    let allow_sample = match frame.packet_id {
                        Some(packet_id) => {
                            if let Some(last) = store.last_packet_id {
                                if packet_id <= last {
                                    false
                                } else {
                                    store.last_packet_id = Some(packet_id);
                                    true
                                }
                            } else {
                                store.last_packet_id = Some(packet_id);
                                true
                            }
                        }
                        None => true,
                    };

                    if allow_sample {
                        let sample = Sample {
                            t_ms: monotonic_ms(start),
                            speed_kph: frame.speed_kph,
                            rpm: frame.rpm,
                            throttle: frame.throttle,
                            brake: frame.brake,
                        };
                        store.samples.push(sample);
                    }
                }

                if frame.source_timestamp_ms.is_some() {
                    store.last_source_timestamp_ms = frame.source_timestamp_ms;
                }
            }
            changed = config_rx.changed() => {
                if changed.is_err() {
                    break;
                }
                let next_config = config_rx.borrow().clone();
                if detect_state.is_none() && next_config.bind_addr != config.bind_addr {
                    match rebind_udp_socket(socket, next_config.bind_addr, active_bind, udp_port)
                        .await
                    {
                        Ok((new_socket, bound_addr, _)) => {
                            socket = new_socket;
                            active_bind = bound_addr;
                        }
                        Err(err) => {
                            warn!(?err, "failed to rebind udp socket");
                            break;
                        }
                    }
                }
                config = next_config;
            }
        }
    }

    Ok(())
}

async fn state_update_task(
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
            (store.state.clone(), store.last_source_timestamp_ms)
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

async fn samples_window_task(
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
            if store.session_state != SessionState::InRace {
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

fn now_epoch_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn monotonic_ms(start: Instant) -> u64 {
    start.elapsed().as_millis() as u64
}

fn next_sequence(sequence: &AtomicU64) -> u64 {
    sequence.fetch_add(1, Ordering::Relaxed) + 1
}

async fn bind_udp_socket(bind_addr: IpAddr, port: u16) -> std::io::Result<tokio::net::UdpSocket> {
    let addr = SocketAddr::new(bind_addr, port);
    let socket = tokio::net::UdpSocket::bind(addr).await?;
    info!(%addr, "udp ingest started");
    Ok(socket)
}

async fn bind_heartbeat_socket(bind_addr: IpAddr) -> std::io::Result<tokio::net::UdpSocket> {
    tokio::net::UdpSocket::bind(SocketAddr::new(bind_addr, 0)).await
}

fn resolve_local_ip_for_target(target: IpAddr) -> std::io::Result<IpAddr> {
    let socket = std::net::UdpSocket::bind(("0.0.0.0", 0))?;
    socket.connect(SocketAddr::new(target, HEARTBEAT_PORT))?;
    Ok(socket.local_addr()?.ip())
}

fn resolve_default_route_ip() -> std::io::Result<IpAddr> {
    let socket = std::net::UdpSocket::bind(("0.0.0.0", 0))?;
    socket.connect(("1.1.1.1", 80))?;
    Ok(socket.local_addr()?.ip())
}

fn fallback_local_ip(config: &UdpConfig) -> Option<IpAddr> {
    match config.bind_addr {
        IpAddr::V4(addr) if !addr.is_loopback() && !addr.is_unspecified() => {
            Some(IpAddr::V4(addr))
        }
        _ => preferred_private_ipv4().or_else(|| resolve_default_route_ip().ok()),
    }
}

fn resolve_broadcast_bind_ip(config: &UdpConfig, pending_detect: bool) -> Option<IpAddr> {
    if !pending_detect {
        return None;
    }
    fallback_local_ip(config)
}

fn preferred_private_ipv4() -> Option<IpAddr> {
    let ifaces = get_if_addrs().ok()?;
    for iface in ifaces {
        if let if_addrs::IfAddr::V4(v4) = iface.addr {
            let ip = v4.ip;
            if is_private_ipv4(ip) && !ip.is_loopback() && !ip.is_link_local() {
                return Some(IpAddr::V4(ip));
            }
        }
    }
    None
}

fn is_private_ipv4(ip: Ipv4Addr) -> bool {
    let octets = ip.octets();
    match octets {
        [10, ..] => true,
        [172, second, ..] if (16..=31).contains(&second) => true,
        [192, 168, ..] => true,
        _ => false,
    }
}

async fn heartbeat_task(
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

async fn rebind_udp_socket(
    socket: tokio::net::UdpSocket,
    target: IpAddr,
    fallback: IpAddr,
    port: u16,
) -> std::io::Result<(tokio::net::UdpSocket, IpAddr, bool)> {
    drop(socket);
    let target_addr = SocketAddr::new(target, port);
    match tokio::net::UdpSocket::bind(target_addr).await {
        Ok(new_socket) => {
            info!(addr = %target_addr, "udp ingest started");
            Ok((new_socket, target, false))
        }
        Err(err) => {
            warn!(?err, addr = %target_addr, "udp rebind failed, restoring");
            let fallback_addr = SocketAddr::new(fallback, port);
            let fallback_socket = tokio::net::UdpSocket::bind(fallback_addr).await?;
            info!(addr = %fallback_addr, "udp ingest started");
            Ok((fallback_socket, fallback, true))
        }
    }
}
