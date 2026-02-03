// Downsampled telemetry sample used in rolling windows.

use serde::Serialize;

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
