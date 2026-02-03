// HTTP response payload types.

use std::net::IpAddr;

use serde::Serialize;

use crate::app::DetectStatus;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
}

#[derive(Serialize)]
pub struct DetectStartResponse {
    pub id: u64,
    pub status: DetectStatus,
    pub timeout_ms: u64,
}

#[derive(Serialize)]
pub struct DetectStatusResponse {
    pub id: u64,
    pub status: DetectStatus,
    pub ps5_ip: Option<IpAddr>,
}

#[derive(Serialize)]
pub struct DemoStatusResponse {
    pub active: bool,
    pub path: Option<String>,
}

#[derive(Serialize)]
pub struct MetaCarResponse {
    pub id: i32,
    pub name: Option<String>,
    pub manufacturer: Option<String>,
}

#[derive(Serialize)]
pub struct MetaTrackResponse {
    pub id: i32,
    pub name: Option<String>,
    pub base_id: Option<i32>,
    pub layout_number: Option<i32>,
    pub is_reverse: Option<bool>,
}

#[derive(Serialize)]
pub struct MetaTrackGeometryResponse {
    pub id: i32,
    pub has_geometry: bool,
    pub path: Option<String>,
}

#[derive(Serialize)]
pub struct MetaTrackGeometrySvgResponse {
    pub id: i32,
    pub exists: bool,
    pub view_box: Option<String>,
    pub path_d: Option<String>,
    pub points_count: Option<usize>,
    pub simplified: Option<bool>,
}

#[derive(Serialize)]
pub struct MetaCurrentResponse {
    pub car_id: Option<i32>,
    pub car_name: Option<String>,
    pub car_manufacturer: Option<String>,
    pub track_id: Option<i32>,
    pub track_name: Option<String>,
    pub has_geometry: bool,
}

#[derive(Serialize)]
pub struct DebugTelemetryResponse {
    pub timestamp_ms: u64,
    pub session: DebugSession,
    pub powertrain: DebugPowertrain,
    pub fluids: DebugFluids,
    pub tyres: DebugTyres,
    pub wheels: DebugWheels,
    pub chassis: DebugChassis,
    pub gears: DebugGears,
    pub dynamics: DebugDynamics,
    pub flags: DebugFlags,
    pub raw: DebugRaw,
}

#[derive(Serialize)]
pub struct DebugSession {
    pub in_race: Option<bool>,
    pub is_paused: Option<bool>,
    pub packet_id: Option<i32>,
    pub time_on_track_ms: Option<i32>,
    pub current_lap: Option<i16>,
    pub total_laps: Option<i16>,
    pub best_lap_ms: Option<i32>,
    pub last_lap_ms: Option<i32>,
    pub current_position: Option<i16>,
    pub total_positions: Option<i16>,
    pub car_id: Option<i32>,
    pub track_id: Option<i32>,
}

#[derive(Serialize)]
pub struct DebugPowertrain {
    pub speed_kph: Option<f32>,
    pub rpm: Option<f32>,
    pub rpm_rev_warning: Option<u16>,
    pub rpm_rev_limiter: Option<u16>,
    pub gear: Option<i8>,
    pub gear_raw: Option<u8>,
    pub suggested_gear: Option<u8>,
    pub throttle: Option<f32>,
    pub brake: Option<f32>,
    pub clutch: Option<f32>,
    pub clutch_engaged: Option<f32>,
    pub rpm_after_clutch: Option<f32>,
    pub boost_kpa: Option<f32>,
    pub estimated_speed_kph: Option<f32>,
}

#[derive(Serialize)]
pub struct DebugFluids {
    pub fuel_l: Option<f32>,
    pub fuel_capacity_l: Option<f32>,
    pub oil_temp_c: Option<f32>,
    pub water_temp_c: Option<f32>,
    pub oil_pressure_kpa: Option<f32>,
}

#[derive(Serialize)]
pub struct DebugTyres {
    pub temp_fl_c: Option<f32>,
    pub temp_fr_c: Option<f32>,
    pub temp_rl_c: Option<f32>,
    pub temp_rr_c: Option<f32>,
    pub tyre_diameter_fl_m: Option<f32>,
    pub tyre_diameter_fr_m: Option<f32>,
    pub tyre_diameter_rl_m: Option<f32>,
    pub tyre_diameter_rr_m: Option<f32>,
}

#[derive(Serialize)]
pub struct DebugWheels {
    pub wheel_speed_fl: Option<f32>,
    pub wheel_speed_fr: Option<f32>,
    pub wheel_speed_rl: Option<f32>,
    pub wheel_speed_rr: Option<f32>,
    pub tyre_speed_fl_kph: Option<f32>,
    pub tyre_speed_fr_kph: Option<f32>,
    pub tyre_speed_rl_kph: Option<f32>,
    pub tyre_speed_rr_kph: Option<f32>,
    pub tyre_slip_ratio_fl: Option<f32>,
    pub tyre_slip_ratio_fr: Option<f32>,
    pub tyre_slip_ratio_rl: Option<f32>,
    pub tyre_slip_ratio_rr: Option<f32>,
}

#[derive(Serialize)]
pub struct DebugChassis {
    pub ride_height_mm: Option<f32>,
    pub suspension_fl: Option<f32>,
    pub suspension_fr: Option<f32>,
    pub suspension_rl: Option<f32>,
    pub suspension_rr: Option<f32>,
}

#[derive(Serialize)]
pub struct DebugGears {
    pub gear_ratio_1: Option<f32>,
    pub gear_ratio_2: Option<f32>,
    pub gear_ratio_3: Option<f32>,
    pub gear_ratio_4: Option<f32>,
    pub gear_ratio_5: Option<f32>,
    pub gear_ratio_6: Option<f32>,
    pub gear_ratio_7: Option<f32>,
    pub gear_ratio_8: Option<f32>,
    pub gear_ratio_unknown: Option<f32>,
}

#[derive(Serialize)]
pub struct DebugDynamics {
    pub pos_x: Option<f32>,
    pub pos_y: Option<f32>,
    pub pos_z: Option<f32>,
    pub vel_x: Option<f32>,
    pub vel_y: Option<f32>,
    pub vel_z: Option<f32>,
    pub angular_vel_x: Option<f32>,
    pub angular_vel_y: Option<f32>,
    pub angular_vel_z: Option<f32>,
    pub accel_long: Option<f32>,
    pub accel_lat: Option<f32>,
    pub yaw_rate: Option<f32>,
    pub pitch: Option<f32>,
    pub roll: Option<f32>,
    pub rotation_yaw: Option<f32>,
    pub rotation_extra: Option<f32>,
}

#[derive(Serialize)]
pub struct DebugFlags {
    pub flags_8e: Option<u8>,
    pub flags_8f: Option<u8>,
    pub flags_93: Option<u8>,
    pub unknown_0x94: Option<f32>,
    pub unknown_0x98: Option<f32>,
    pub unknown_0x9c: Option<f32>,
    pub unknown_0xa0: Option<f32>,
    pub unknown_0xd4: Option<f32>,
    pub unknown_0xd8: Option<f32>,
    pub unknown_0xdc: Option<f32>,
    pub unknown_0xe0: Option<f32>,
    pub unknown_0xe4: Option<f32>,
    pub unknown_0xe8: Option<f32>,
    pub unknown_0xec: Option<f32>,
    pub unknown_0xf0: Option<f32>,
}

#[derive(Serialize)]
pub struct DebugRaw {
    pub encrypted_len: Option<usize>,
    pub decrypted_len: Option<usize>,
    pub encrypted_hex: Option<String>,
    pub decrypted_hex: Option<String>,
    pub source_ip: Option<IpAddr>,
    pub captured_at_ms: Option<u64>,
    pub last_telemetry_ms: Option<u64>,
    pub last_source_timestamp_ms: Option<u64>,
}
