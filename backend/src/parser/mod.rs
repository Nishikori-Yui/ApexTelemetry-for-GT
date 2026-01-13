// GT7 telemetry parser.
// Invariants: uncertain fields are represented as Option or Unknown; no guessing.

use crate::model::TelemetryFrame;

pub fn parse_telemetry(payload: &[u8]) -> Option<TelemetryFrame> {
    let rpm = read_f32(payload, 0x3C);
    let speed_ms = read_f32(payload, 0x4C);
    let speed_kph = speed_ms.map(|value| value * 3.6);
    let throttle = read_u8(payload, 0x91).map(|value| value as f32 / 255.0);
    let brake = read_u8(payload, 0x92).map(|value| value as f32 / 255.0);
    let gear_raw = read_u8(payload, 0x90).map(|value| value & 0x0F);
    let gear = gear_raw.map(|value| if value == 0 { -1 } else { value as i8 });
    let packet_id = read_i32(payload, 0x70);
    let current_lap = read_i16(payload, 0x74);
    let total_laps = read_i16(payload, 0x76);
    let best_lap_ms = read_i32(payload, 0x78);
    let last_lap_ms = read_i32(payload, 0x7C);
    let time_on_track_ms = read_i32(payload, 0x80);
    let flags = read_u8(payload, 0x8E);
    let in_race = flags.map(|value| (value & 0b0000_0001) != 0);
    let is_paused = flags.map(|value| (value & 0b0000_0010) != 0);

    let has_any = rpm.is_some()
        || speed_kph.is_some()
        || throttle.is_some()
        || brake.is_some()
        || gear.is_some()
        || packet_id.is_some()
        || current_lap.is_some()
        || total_laps.is_some()
        || best_lap_ms.is_some()
        || last_lap_ms.is_some()
        || time_on_track_ms.is_some()
        || in_race.is_some()
        || is_paused.is_some();

    if !has_any {
        return None;
    }

    Some(TelemetryFrame {
        speed_kph,
        rpm,
        gear,
        throttle,
        brake,
        in_race,
        is_paused,
        packet_id,
        current_lap,
        total_laps,
        best_lap_ms,
        last_lap_ms,
        time_on_track_ms,
        source_timestamp_ms: None,
    })
}

fn read_f32(payload: &[u8], offset: usize) -> Option<f32> {
    let bytes = payload.get(offset..offset + 4)?;
    Some(f32::from_le_bytes(bytes.try_into().ok()?))
}

fn read_u8(payload: &[u8], offset: usize) -> Option<u8> {
    payload.get(offset).copied()
}

fn read_i16(payload: &[u8], offset: usize) -> Option<i16> {
    let bytes = payload.get(offset..offset + 2)?;
    Some(i16::from_le_bytes(bytes.try_into().ok()?))
}

fn read_i32(payload: &[u8], offset: usize) -> Option<i32> {
    let bytes = payload.get(offset..offset + 4)?;
    Some(i32::from_le_bytes(bytes.try_into().ok()?))
}
