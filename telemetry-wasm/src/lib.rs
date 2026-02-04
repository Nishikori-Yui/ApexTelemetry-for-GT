use serde::Serialize;
use telemetry_core::packet::parse_packet_meta;
use telemetry_core::{crypto, parser};
use telemetry_core::session::SessionTracker;
use wasm_bindgen::prelude::*;

#[derive(Serialize)]
struct DemoFrame {
    t_ms: u64,
    state: telemetry_core::model::State,
}

#[wasm_bindgen]
pub fn decode_demo_bin(
    data: &[u8],
    fixed_track_id: Option<i32>,
    fixed_car_id: Option<i32>,
) -> Result<JsValue, JsValue> {
    let mut offset = 0usize;
    let mut frames = Vec::new();
    let mut session = SessionTracker::new();

    if let Some(car_id) = fixed_car_id {
        session.set_car_id(Some(car_id));
    }
    if let Some(track_id) = fixed_track_id {
        session.set_track_id(Some(track_id));
    }

    while offset + 12 <= data.len() {
        let t_ms = u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap());
        let len = u32::from_le_bytes(data[offset + 8..offset + 12].try_into().unwrap()) as usize;
        offset += 12;
        if len == 0 {
            continue;
        }
        if offset + len > data.len() {
            return Err(JsValue::from_str("demo bin truncated"));
        }
        let encrypted = &data[offset..offset + len];
        offset += len;

        let payload = match crypto::decrypt_packet(encrypted) {
            Some(payload) => payload,
            None => continue,
        };
        let frame = match parser::parse_telemetry(&payload) {
            Some(frame) => frame,
            None => continue,
        };
        let packet_meta = parse_packet_meta(&payload);
        session.apply_frame(&frame, t_ms, packet_meta.car_id);
        if let Some(car_id) = fixed_car_id {
            session.set_car_id(Some(car_id));
        }
        if let Some(track_id) = fixed_track_id {
            session.set_track_id(Some(track_id));
        }
        frames.push(DemoFrame {
            t_ms,
            state: session.state.clone(),
        });
    }

    if offset != data.len() {
        return Err(JsValue::from_str("demo bin has trailing bytes"));
    }
    if frames.is_empty() {
        return Err(JsValue::from_str("demo bin decoded zero frames"));
    }

    serde_wasm_bindgen::to_value(&frames)
        .map_err(|err| JsValue::from_str(&err.to_string()))
}
