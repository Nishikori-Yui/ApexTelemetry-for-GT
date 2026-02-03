// Shared constants for server timing, protocol, and paths.

pub const SCHEMA_VERSION: &str = "1.0";
pub const STATE_INTERVAL_MS: u64 = 50;
pub const WINDOW_INTERVAL_MS: u64 = 250;
pub const WINDOW_DURATION_MS: u64 = 5_000;
pub const WINDOW_STRIDE_MS: u64 = 50;
pub const SAMPLE_BUFFER_CAP: usize = 600;
pub const RAW_PACKET_HISTORY: usize = 5;
pub const HEARTBEAT_PORT: u16 = 33739;
pub const HEARTBEAT_INTERVAL_SECS: u64 = 1;
pub const HEARTBEAT_BYTE: u8 = 0x41;
pub const DEMO_DIR: &str = "demo";
pub const DEMO_FILE: &str = "demo_race.bin";
