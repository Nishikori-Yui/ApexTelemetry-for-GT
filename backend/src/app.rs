// Application state and shared data structures for the server.

use std::collections::{HashMap, VecDeque};
use std::net::{IpAddr, Ipv4Addr};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::io::BufWriter;
use tokio::sync::{broadcast, mpsc, oneshot, watch, Mutex, RwLock};
use tokio::time::Instant;

use crate::constants::{RAW_PACKET_HISTORY, SAMPLE_BUFFER_CAP};
use crate::buffers::RingBuffer;
use crate::meta::MetadataStore;
use crate::model::Sample;
use telemetry_core::session::SessionTracker;
pub use telemetry_core::session::SessionState;

#[derive(Clone)]
pub struct AppState {
    pub tx: broadcast::Sender<String>,
    pub sequence: Arc<AtomicU64>,
    pub start_instant: Instant,
    pub udp_config_tx: watch::Sender<UdpConfig>,
    pub detect_tx: mpsc::Sender<DetectCommand>,
    pub detect_store: Arc<RwLock<DetectStore>>,
    pub detect_sequence: Arc<AtomicU64>,
    pub store: Arc<RwLock<TelemetryStore>>,
    pub meta: Arc<MetadataStore>,
    pub demo_active: Arc<AtomicBool>,
    pub demo_state: Arc<Mutex<DemoState>>,
    pub record_state: Arc<Mutex<RecordState>>,
    pub data_dir: PathBuf,
}

pub struct TelemetryStore {
    pub session: SessionTracker,
    pub samples: RingBuffer<Sample>,
    pub last_packet_id: Option<i32>,
    pub last_source_timestamp_ms: Option<u64>,
    pub last_telemetry_ms: Option<u64>,
    pub last_packet_len: Option<usize>,
    pub last_payload_len: Option<usize>,
    pub last_source_ip: Option<IpAddr>,
    pub raw_packets: VecDeque<RawPacketSnapshot>,
}

impl TelemetryStore {
    pub fn new() -> Self {
        Self {
            session: SessionTracker::new(),
            samples: RingBuffer::new(SAMPLE_BUFFER_CAP),
            last_packet_id: None,
            last_source_timestamp_ms: None,
            last_telemetry_ms: None,
            last_packet_len: None,
            last_payload_len: None,
            last_source_ip: None,
            raw_packets: VecDeque::with_capacity(RAW_PACKET_HISTORY),
        }
    }
}

pub struct RawPacketSnapshot {
    pub captured_at_ms: u64,
    pub source_ip: Option<IpAddr>,
    pub encrypted: Vec<u8>,
    pub decrypted: Vec<u8>,
}

pub struct PacketInfo {
    pub packet_len: Option<usize>,
    pub payload_len: Option<usize>,
    pub source_ip: Option<IpAddr>,
    pub raw_snapshot: Option<RawPacketSnapshot>,
}

#[derive(Default)]
pub struct DemoState {
    pub active: bool,
    pub path: Option<PathBuf>,
    pub cancel: Option<oneshot::Sender<()>>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RecordMode {
    Idle,
    Armed,
    Recording,
}

impl RecordMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            RecordMode::Idle => "idle",
            RecordMode::Armed => "armed",
            RecordMode::Recording => "recording",
        }
    }
}

pub struct RecordState {
    pub mode: RecordMode,
    pub path: Option<PathBuf>,
    pub writer: Option<BufWriter<tokio::fs::File>>,
    pub start_ms: Option<u64>,
    pub frames: u64,
}

impl Default for RecordState {
    fn default() -> Self {
        Self {
            mode: RecordMode::Idle,
            path: None,
            writer: None,
            start_ms: None,
            frames: 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UdpConfig {
    pub bind_addr: IpAddr,
    pub ps5_ip: Option<IpAddr>,
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
pub enum DetectStatus {
    Pending,
    Found,
    Timeout,
    Error,
    Cancelled,
}

#[derive(Clone, Debug)]
pub struct DetectSession {
    pub id: u64,
    pub status: DetectStatus,
    pub ps5_ip: Option<IpAddr>,
    pub timeout_ms: u64,
}

#[derive(Clone, Debug)]
pub struct DetectEvent {
    pub id: u64,
    pub status: DetectStatus,
}

#[derive(Default)]
pub struct DetectStore {
    pub sessions: HashMap<u64, DetectSession>,
    pub active_id: Option<u64>,
    pub last_event: Option<DetectEvent>,
}

pub struct DetectCommand {
    pub id: u64,
    pub timeout_ms: u64,
}

pub struct DetectState {
    pub id: u64,
    pub deadline: Instant,
}

#[derive(Clone, Debug, PartialEq)]
pub enum HeartbeatMode {
    Stop,
    Broadcast,
    Unicast(IpAddr),
}
