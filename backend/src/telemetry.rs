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
        let should_stop_record = next_state == SessionState::NotInRace;

        if next_state != previous_state {
            if next_state == SessionState::InRace && previous_state == SessionState::NotInRace {
                store.session_index += 1;
                store.samples.clear();
                store.last_packet_id = None;
                store.last_current_lap = None;
                store.last_lap_time_ms_recorded = None;
                store.lap_start_mono_ms = Some(now_ms);
                store.lap_pause_started_ms = None;
                store.lap_pause_accum_ms = 0;
                store.fuel_pct_at_lap_start = None;
                store.fuel_consume_history.clear();
                track_detector.reset();
                store.track_id = None;
            } else if next_state == SessionState::NotInRace {
                track_detector.reset();
                store.track_id = None;
                store.lap_start_mono_ms = None;
                store.lap_pause_started_ms = None;
                store.lap_pause_accum_ms = 0;
                store.state.current_lap_time_ms = None;
            }
            store.session_state = next_state;
            info!(
                from = ?previous_state,
                to = ?next_state,
                session_index = store.session_index,
                "session transition"
            );
        }

        store.state.update_from(frame);

        if let Some(last_lap_ms) = frame.last_lap_ms {
            if store.last_lap_time_ms_recorded != Some(last_lap_ms) {
                store.last_lap_time_ms_recorded = Some(last_lap_ms);
                if store.session_state != SessionState::NotInRace {
                    store.lap_start_mono_ms = Some(now_ms);
                    store.lap_pause_started_ms = None;
                    store.lap_pause_accum_ms = 0;
                }
            }
        }
        if store.session_state == SessionState::InRace && store.lap_start_mono_ms.is_none() {
            store.lap_start_mono_ms = Some(now_ms);
        }

        let should_start_record = store.session_state == SessionState::InRace;

        match store.session_state {
            SessionState::Paused => {
                if store.lap_pause_started_ms.is_none() {
                    store.lap_pause_started_ms = Some(now_ms);
                }
            }
            SessionState::InRace => {
                if let Some(pause_start) = store.lap_pause_started_ms.take() {
                    store.lap_pause_accum_ms = store
                        .lap_pause_accum_ms
                        .saturating_add(now_ms.saturating_sub(pause_start));
                }
            }
            SessionState::NotInRace => {
                store.lap_pause_started_ms = None;
                store.lap_pause_accum_ms = 0;
            }
        }

        if let Some(lap_start) = store.lap_start_mono_ms {
            let mut elapsed = now_ms.saturating_sub(lap_start);
            elapsed = elapsed.saturating_sub(store.lap_pause_accum_ms);
            if let Some(pause_start) = store.lap_pause_started_ms {
                elapsed = elapsed.saturating_sub(now_ms.saturating_sub(pause_start));
            }
            let safe_elapsed = elapsed.min(i32::MAX as u64) as i32;
            store.state.current_lap_time_ms = Some(safe_elapsed);
        } else {
            store.state.current_lap_time_ms = None;
        }

        let current_fuel_pct = match (frame.fuel_l, frame.fuel_capacity_l) {
            (Some(fuel), Some(cap)) if cap > 0.0 => Some((fuel / cap) * 100.0),
            _ => None,
        };

        if let Some(current_lap) = frame.current_lap {
            let lap_changed = store
                .last_current_lap
                .map(|prev| prev != current_lap)
                .unwrap_or(true);
            if lap_changed && store.last_current_lap.is_some() {
                let valid_lap = store
                    .last_lap_time_ms_recorded
                    .map(|t| t > 0)
                    .unwrap_or(false);

                if valid_lap {
                    if let (Some(start_pct), Some(end_pct)) =
                        (store.fuel_pct_at_lap_start, current_fuel_pct)
                    {
                        let consume = (start_pct - end_pct).max(0.0);
                        if consume > 0.0 {
                            if store.fuel_consume_history.len() >= 3 {
                                store.fuel_consume_history.pop_back();
                            }
                            store.fuel_consume_history.push_front(consume);
                        }
                    }
                }
            }
            if lap_changed || store.fuel_pct_at_lap_start.is_none() {
                store.fuel_pct_at_lap_start = current_fuel_pct;
            }

            store.last_current_lap = Some(current_lap);
        }

        if !store.fuel_consume_history.is_empty() {
            let sum: f32 = store.fuel_consume_history.iter().sum();
            let avg = sum / store.fuel_consume_history.len() as f32;
            store.state.avg_fuel_consume_pct_per_lap = Some(avg);
            if let Some(fuel_pct) = current_fuel_pct {
                if avg > 0.0 {
                    store.state.fuel_laps_remaining = Some(fuel_pct / avg);
                }
            }
        }

        if let Some(car_id) = packet_meta.car_id {
            store.car_id = Some(car_id);
        }
        store.track_id = track_detector.update(
            store.session_state == SessionState::InRace,
            frame.is_paused.unwrap_or(false),
            frame.current_lap,
            packet_meta.position_xz,
            meta.track_bounds(),
        );
        store.state.car_id = store.car_id;
        store.state.track_id = store.track_id;

        store.state.pos_x = frame.pos_x;
        store.state.pos_y = frame.pos_y;
        store.state.pos_z = frame.pos_z;

        store.state.vel_x = frame.vel_x;
        store.state.vel_y = frame.vel_y;
        store.state.vel_z = frame.vel_z;

        store.state.rotation_yaw = frame.rotation_yaw;

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
        (should_stop_record, should_start_record)
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
