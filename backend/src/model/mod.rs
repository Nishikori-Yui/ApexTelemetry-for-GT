// Core data models for state, samples, and lap/session events.
// Invariants: state, samples, and lap/session layers are strictly separated.

use serde::Serialize;

#[derive(Clone, Debug, Default, Serialize)]
pub struct State {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed_kph: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rpm: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gear: Option<i8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub throttle: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brake: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_race: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_paused: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub packet_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_lap: Option<i16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_laps: Option<i16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub best_lap_ms: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_lap_ms: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_on_track_ms: Option<i32>,
}

impl State {
    pub fn update_from(&mut self, frame: &TelemetryFrame) {
        if frame.speed_kph.is_some() {
            self.speed_kph = frame.speed_kph;
        }
        if frame.rpm.is_some() {
            self.rpm = frame.rpm;
        }
        if frame.gear.is_some() {
            self.gear = frame.gear;
        }
        if frame.throttle.is_some() {
            self.throttle = frame.throttle;
        }
        if frame.brake.is_some() {
            self.brake = frame.brake;
        }
        if frame.in_race.is_some() {
            self.in_race = frame.in_race;
        }
        if frame.is_paused.is_some() {
            self.is_paused = frame.is_paused;
        }
        if frame.packet_id.is_some() {
            self.packet_id = frame.packet_id;
        }
        if frame.current_lap.is_some() {
            self.current_lap = frame.current_lap;
        }
        if frame.total_laps.is_some() {
            self.total_laps = frame.total_laps;
        }
        if frame.best_lap_ms.is_some() {
            self.best_lap_ms = frame.best_lap_ms;
        }
        if frame.last_lap_ms.is_some() {
            self.last_lap_ms = frame.last_lap_ms;
        }
        if frame.time_on_track_ms.is_some() {
            self.time_on_track_ms = frame.time_on_track_ms;
        }
    }

    pub fn is_empty(&self) -> bool {
        self.speed_kph.is_none()
            && self.rpm.is_none()
            && self.gear.is_none()
            && self.throttle.is_none()
            && self.brake.is_none()
            && self.in_race.is_none()
            && self.is_paused.is_none()
            && self.packet_id.is_none()
            && self.current_lap.is_none()
            && self.total_laps.is_none()
            && self.best_lap_ms.is_none()
            && self.last_lap_ms.is_none()
            && self.time_on_track_ms.is_none()
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Sample {
    pub t_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed_kph: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rpm: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub throttle: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brake: Option<f32>,
}

#[derive(Clone, Debug)]
pub struct TelemetryFrame {
    pub speed_kph: Option<f32>,
    pub rpm: Option<f32>,
    pub gear: Option<i8>,
    pub throttle: Option<f32>,
    pub brake: Option<f32>,
    pub in_race: Option<bool>,
    pub is_paused: Option<bool>,
    pub packet_id: Option<i32>,
    pub current_lap: Option<i16>,
    pub total_laps: Option<i16>,
    pub best_lap_ms: Option<i32>,
    pub last_lap_ms: Option<i32>,
    pub time_on_track_ms: Option<i32>,
    pub source_timestamp_ms: Option<u64>,
}
