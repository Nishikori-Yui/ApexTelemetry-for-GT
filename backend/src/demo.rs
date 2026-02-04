// Demo playback utilities and data path resolution.

use std::env;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use tokio::io::{AsyncReadExt, BufReader};
use tokio::sync::{oneshot, RwLock};
use tokio::time::{self, Instant};

use crate::app::TelemetryStore;
use crate::constants::{DEMO_DIR, DEMO_FILE};
use crate::meta::{self, MetadataStore, PacketMeta, TrackDetector};
use crate::model::TelemetryFrame;
use crate::telemetry::apply_frame;
use crate::utils::monotonic_ms;
use telemetry_core::crypto;
use telemetry_core::parser;

pub fn demo_default_path(data_dir: &Path) -> PathBuf {
    data_dir.join(DEMO_DIR).join(DEMO_FILE)
}

pub fn resolve_demo_path(data_dir: &Path) -> PathBuf {
    let primary = demo_default_path(data_dir);
    if primary.is_file() {
        return primary;
    }
    let fallback = PathBuf::from("../data").join(DEMO_DIR).join(DEMO_FILE);
    if fallback.is_file() {
        return fallback;
    }
    primary
}

pub fn resolve_data_dir() -> PathBuf {
    if let Ok(value) = env::var("APEXTELEMETRY_DATA_DIR") {
        return PathBuf::from(value);
    }
    let local = PathBuf::from("./data");
    if local.is_dir() {
        return local;
    }
    let parent = PathBuf::from("../data");
    if parent.is_dir() {
        return parent;
    }
    local
}

pub async fn reset_store_for_demo(store: &Arc<RwLock<TelemetryStore>>) {
    let mut store = store.write().await;
    store.session.reset_for_demo();
    store.samples.clear();
    store.last_packet_id = None;
    store.raw_packets.clear();
    store.last_packet_len = None;
    store.last_payload_len = None;
    store.last_source_ip = None;
    store.last_telemetry_ms = None;
    store.last_source_timestamp_ms = None;
}

pub async fn demo_playback_loop(
    path: PathBuf,
    store: Arc<RwLock<TelemetryStore>>,
    meta: Arc<MetadataStore>,
    start: Instant,
    mut cancel: oneshot::Receiver<()>,
) -> std::io::Result<()> {
    let mut track_detector = TrackDetector::new();
    let mut first_pass = true;

    loop {
        if !first_pass {
            reset_store_for_demo(&store).await;
            track_detector.reset();
        } else {
            first_pass = false;
        }

        let file = tokio::fs::File::open(&path).await?;
        let mut reader = BufReader::new(file);
        let mut last_offset = 0u64;
        let mut has_record = false;

        loop {
            let mut header = [0u8; 12];
            let read = tokio::select! {
                _ = &mut cancel => return Ok(()),
                read = reader.read_exact(&mut header) => read,
            };
            match read {
                Ok(_) => {}
                Err(err) if err.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(err) => return Err(err),
            }
            let offset_ms = u64::from_le_bytes(header[0..8].try_into().unwrap());
            let len = u32::from_le_bytes(header[8..12].try_into().unwrap()) as usize;
            if len == 0 {
                continue;
            }
            let mut packet = vec![0u8; len];
            let read = tokio::select! {
                _ = &mut cancel => return Ok(()),
                read = reader.read_exact(&mut packet) => read,
            };
            match read {
                Ok(_) => {}
                Err(err) if err.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(err) => return Err(err),
            }

            has_record = true;
            let delay_ms = offset_ms.saturating_sub(last_offset);
            if delay_ms > 0 {
                tokio::select! {
                    _ = &mut cancel => return Ok(()),
                    _ = time::sleep(std::time::Duration::from_millis(delay_ms)) => {}
                }
            }

            let payload = match crypto::decrypt_packet(&packet) {
                Some(payload) => payload,
                None => {
                    last_offset = offset_ms;
                    continue;
                }
            };
            let frame: TelemetryFrame = match parser::parse_telemetry(&payload) {
                Some(frame) => frame,
                None => {
                    last_offset = offset_ms;
                    continue;
                }
            };

            let now_ms = monotonic_ms(start);
            let position_xz = match (frame.pos_x, frame.pos_z) {
                (Some(x), Some(z)) => Some((x, z)),
                _ => None,
            };
            let packet_meta = meta::parse_packet_meta(&payload);
            let packet_meta = PacketMeta {
                car_id: packet_meta.car_id,
                position_xz,
            };
            apply_frame(
                &store,
                &meta,
                &mut track_detector,
                &frame,
                &packet_meta,
                now_ms,
                None,
                None,
            )
            .await;
            last_offset = offset_ms;
        }

        if !has_record {
            tokio::select! {
                _ = &mut cancel => return Ok(()),
                _ = time::sleep(std::time::Duration::from_millis(1000)) => {}
            }
        }
    }
}
