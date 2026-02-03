// Raw packet recording helpers and status serialization.

use std::sync::Arc;

use serde::Serialize;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;

use crate::app::{RecordMode, RecordState};

#[derive(Serialize)]
pub struct RecordStatusResponse {
    pub mode: &'static str,
    pub active: bool,
    pub armed: bool,
    pub path: Option<String>,
    pub frames: u64,
}

pub fn record_status_snapshot(state: &RecordState) -> RecordStatusResponse {
    RecordStatusResponse {
        mode: state.mode.as_str(),
        active: state.mode == RecordMode::Recording,
        armed: state.mode == RecordMode::Armed,
        path: state
            .path
            .as_ref()
            .map(|path| path.to_string_lossy().to_string()),
        frames: state.frames,
    }
}

pub async fn stop_recording_internal(
    record_state: &Arc<Mutex<RecordState>>,
) -> RecordStatusResponse {
    let mut state = record_state.lock().await;
    if state.mode != RecordMode::Recording {
        return record_status_snapshot(&state);
    }
    if let Some(writer) = state.writer.as_mut() {
        let _ = writer.flush().await;
    }
    state.mode = RecordMode::Idle;
    state.writer = None;
    state.start_ms = None;
    record_status_snapshot(&state)
}

pub async fn maybe_start_recording(record_state: &Arc<Mutex<RecordState>>, now_ms: u64) {
    let path = {
        let state = record_state.lock().await;
        if state.mode != RecordMode::Armed {
            return;
        }
        state.path.clone()
    };
    let path = match path {
        Some(path) => path,
        None => {
            let mut state = record_state.lock().await;
            state.mode = RecordMode::Idle;
            return;
        }
    };

    let file = match tokio::fs::File::create(&path).await {
        Ok(file) => file,
        Err(err) => {
            tracing::warn!(?err, path = %path.display(), "failed to create demo record file");
            let mut state = record_state.lock().await;
            state.mode = RecordMode::Idle;
            state.writer = None;
            state.start_ms = None;
            return;
        }
    };

    let mut state = record_state.lock().await;
    if state.mode != RecordMode::Armed {
        return;
    }
    state.writer = Some(tokio::io::BufWriter::new(file));
    state.start_ms = Some(now_ms);
    state.frames = 0;
    state.mode = RecordMode::Recording;
}

pub async fn record_raw_packet(record_state: &Arc<Mutex<RecordState>>, now_ms: u64, encrypted: &[u8]) {
    let mut state = record_state.lock().await;
    if state.mode != RecordMode::Recording {
        return;
    }
    let start_ms = state.start_ms.get_or_insert(now_ms);
    let offset_ms = now_ms.saturating_sub(*start_ms);
    let len = encrypted.len().min(u32::MAX as usize) as u32;
    if let Some(writer) = state.writer.as_mut() {
        if writer.write_all(&offset_ms.to_le_bytes()).await.is_err() {
            state.mode = RecordMode::Idle;
            state.writer = None;
            state.start_ms = None;
            return;
        }
        if writer.write_all(&len.to_le_bytes()).await.is_err() {
            state.mode = RecordMode::Idle;
            state.writer = None;
            state.start_ms = None;
            return;
        }
        if writer.write_all(&encrypted[..len as usize]).await.is_err() {
            state.mode = RecordMode::Idle;
            state.writer = None;
            state.start_ms = None;
            return;
        }
        state.frames = state.frames.saturating_add(1);
    }
}
