// Telemetry state application logic shared by UDP ingest and demo playback.

use std::sync::Arc;

use tokio::sync::{Mutex, RwLock};
use tracing::info;

use crate::app::{PacketInfo, RecordState, SessionState, TelemetryStore};
use crate::constants::RAW_PACKET_HISTORY;
use crate::meta::{MetadataStore, PacketMeta, TrackDetector};
use crate::model::{Sample, TelemetryFrame};
use crate::recording::{maybe_start_recording, stop_recording_internal};

pub async fn apply_frame(
    store: &Arc<RwLock<TelemetryStore>>,
    meta: &Arc<MetadataStore>,
    track_detector: &mut TrackDetector,
    frame: &TelemetryFrame,
    packet_meta: &PacketMeta,
    now_ms: u64,
    packet_info: Option<PacketInfo>,
    record_state: Option<&Arc<Mutex<RecordState>>>,
) {
    let (should_stop_record, should_start_record) = {
        let mut store = store.write().await;
        store.last_telemetry_ms = Some(now_ms);

        if let Some(info) = packet_info {
            if let Some(packet_len) = info.packet_len {
                store.last_packet_len = Some(packet_len);
            }
            if let Some(payload_len) = info.payload_len {
                store.last_payload_len = Some(payload_len);
            }
            if let Some(source_ip) = info.source_ip {
                store.last_source_ip = Some(source_ip);
            }
            if let Some(snapshot) = info.raw_snapshot {
                if store.raw_packets.len() >= RAW_PACKET_HISTORY {
                    store.raw_packets.pop_front();
                }
                store.raw_packets.push_back(snapshot);
            }
        }

        let events = store.session.apply_frame(frame, now_ms, packet_meta.car_id);

        if let Some(transition) = events.transition {
            if transition.to == SessionState::InRace && transition.from == SessionState::NotInRace {
                store.samples.clear();
                store.last_packet_id = None;
                track_detector.reset();
            } else if transition.to == SessionState::NotInRace {
                track_detector.reset();
            }
            info!(
                from = ?transition.from,
                to = ?transition.to,
                session_index = store.session.session_index,
                "session transition"
            );
        }

        let track_id = track_detector.update(
            store.session.session_state == SessionState::InRace,
            frame.is_paused.unwrap_or(false),
            frame.current_lap,
            packet_meta.position_xz,
            meta.track_bounds(),
        );
        store.session.set_track_id(track_id);

        if store.session.session_state == SessionState::InRace {
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
                    t_ms: now_ms,
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
        (events.should_stop_record, events.should_start_record)
    };

    if should_stop_record {
        if let Some(record_state) = record_state {
            let _ = stop_recording_internal(record_state).await;
        }
    }

    if should_start_record {
        if let Some(record_state) = record_state {
            maybe_start_recording(record_state, now_ms).await;
        }
    }
}
