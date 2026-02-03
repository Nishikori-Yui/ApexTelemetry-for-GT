// Minimal telemetry pipeline server for ApexTelemetry for GT.

use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::Arc;

use tokio::sync::{broadcast, mpsc, watch, Mutex, RwLock};
use tokio::time::Instant;
use tracing::{info, warn};

use apextelemetry_for_gt_server::app::{AppState, DetectStore, RecordState, TelemetryStore, UdpConfig};
use apextelemetry_for_gt_server::demo::resolve_data_dir;
use apextelemetry_for_gt_server::http;
use apextelemetry_for_gt_server::meta::MetadataStore;
use apextelemetry_for_gt_server::tasks;
use apextelemetry_for_gt_server::udp;

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

    let data_dir_path = resolve_data_dir();
    let meta = Arc::new(MetadataStore::load(&data_dir_path));

    let store = Arc::new(RwLock::new(TelemetryStore::new()));

    let (tx, _) = broadcast::channel::<String>(256);
    let (udp_config_tx, udp_config_rx) = watch::channel(UdpConfig {
        bind_addr: udp_bind_addr,
        ps5_ip: None,
    });
    let (detect_tx, detect_rx) = mpsc::channel(8);
    let detect_store = Arc::new(RwLock::new(DetectStore::default()));
    let detect_sequence = Arc::new(AtomicU64::new(0));
    let sequence = Arc::new(AtomicU64::new(0));
    let demo_active = Arc::new(AtomicBool::new(false));
    let demo_state = Arc::new(Mutex::new(Default::default()));
    let record_state = Arc::new(Mutex::new(RecordState::default()));
    let start_instant = Instant::now();

    let udp_store = store.clone();
    let udp_meta = meta.clone();
    let udp_start = start_instant;
    let udp_detect_store = detect_store.clone();
    let udp_config_tx_udp = udp_config_tx.clone();
    let udp_demo_active = demo_active.clone();
    let udp_record_state = record_state.clone();
    tokio::spawn(async move {
        if let Err(err) = udp::udp_loop(
            udp_port,
            udp_config_rx,
            udp_config_tx_udp,
            detect_rx,
            udp_detect_store,
            udp_store,
            udp_meta,
            udp_start,
            udp_demo_active,
            udp_record_state,
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
        tasks::state_update_task(state_store, state_tx, state_seq, state_start).await;
    });

    let samples_store = store.clone();
    let samples_tx = tx.clone();
    let samples_seq = sequence.clone();
    let samples_start = start_instant;
    tokio::spawn(async move {
        tasks::samples_window_task(samples_store, samples_tx, samples_seq, samples_start).await;
    });

    let heartbeat_config_rx = udp_config_tx.subscribe();
    let heartbeat_detect_store = detect_store.clone();
    let heartbeat_store = store.clone();
    let heartbeat_start = start_instant;
    tokio::spawn(async move {
        if let Err(err) = tasks::heartbeat_task(
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

    let app_state = AppState {
        tx,
        sequence,
        start_instant,
        udp_config_tx,
        detect_tx,
        detect_store,
        detect_sequence,
        store,
        meta,
        demo_active,
        demo_state,
        record_state,
        data_dir: data_dir_path,
    };

    let app = http::router(app_state);

    info!(%addr, "starting server");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("server failed");
}
