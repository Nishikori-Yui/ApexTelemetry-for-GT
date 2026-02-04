// UDP ingest module.
// Invariants: only binds to localhost by default; raw packets are forwarded without logging payloads.

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{mpsc, watch, Mutex, RwLock};
use tokio::time::{self, Instant};
use tracing::{info, warn};

use crate::app::{
    DetectCommand, DetectEvent, DetectState, DetectStatus, DetectStore, PacketInfo, RawPacketSnapshot,
    RecordState, UdpConfig,
};
use crate::meta::{self, MetadataStore, TrackDetector};
use crate::recording::record_raw_packet;
use crate::telemetry::apply_frame;
use crate::utils::monotonic_ms;
use telemetry_core::crypto;
use telemetry_core::parser;

pub async fn udp_loop(
    udp_port: u16,
    mut config_rx: watch::Receiver<UdpConfig>,
    config_tx: watch::Sender<UdpConfig>,
    mut detect_rx: mpsc::Receiver<DetectCommand>,
    detect_store: Arc<RwLock<DetectStore>>,
    store: Arc<RwLock<crate::app::TelemetryStore>>,
    meta: Arc<MetadataStore>,
    start: Instant,
    demo_active: Arc<AtomicBool>,
    record_state: Arc<Mutex<RecordState>>,
) -> std::io::Result<()> {
    let mut config = config_rx.borrow().clone();
    let mut socket = bind_udp_socket(config.bind_addr, udp_port).await?;
    let mut active_bind = config.bind_addr;
    let mut detect_state: Option<DetectState> = None;
    let mut detect_tick = time::interval(Duration::from_millis(200));
    let mut buf = [0u8; 4096];
    let mut last_inspect_log_ms: u64 = 0;
    let mut track_detector = TrackDetector::new();

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
                if demo_active.load(Ordering::Relaxed) {
                    continue;
                }
                let payload = match crypto::decrypt_packet(&buf[..len]) {
                    Some(payload) => payload,
                    None => continue,
                };
                let frame = match parser::parse_telemetry(&payload) {
                    Some(frame) => frame,
                    None => continue,
                };
                let packet_meta = meta::parse_packet_meta(&payload);
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

                if now_ms.saturating_sub(last_inspect_log_ms) >= 1000 {
                    last_inspect_log_ms = now_ms;
                    let (session_state, session_index) = {
                        let store = store.read().await;
                        (store.session.session_state, store.session.session_index)
                    };
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
                        session_state = ?session_state,
                        session_index,
                        "telemetry inspect"
                    );
                }

                let raw_snapshot = RawPacketSnapshot {
                    captured_at_ms: crate::utils::now_epoch_ms(),
                    source_ip: Some(source.ip()),
                    encrypted: buf[..len].to_vec(),
                    decrypted: payload.clone(),
                };

                let packet_info = PacketInfo {
                    packet_len: Some(len),
                    payload_len: Some(payload.len()),
                    source_ip: Some(source.ip()),
                    raw_snapshot: Some(raw_snapshot),
                };

                apply_frame(
                    &store,
                    &meta,
                    &mut track_detector,
                    &frame,
                    &packet_meta,
                    now_ms,
                    Some(packet_info),
                    Some(&record_state),
                )
                .await;
                record_raw_packet(&record_state, now_ms, &buf[..len]).await;
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

async fn bind_udp_socket(bind_addr: IpAddr, port: u16) -> std::io::Result<tokio::net::UdpSocket> {
    let addr = SocketAddr::new(bind_addr, port);
    let socket = tokio::net::UdpSocket::bind(addr).await?;
    info!(%addr, "udp ingest started");
    Ok(socket)
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
