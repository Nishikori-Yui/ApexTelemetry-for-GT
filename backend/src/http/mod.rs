// HTTP handlers and routing.

use std::sync::atomic::Ordering;

use axum::extract::State as AxumState;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use serde_json::json;
use tracing::info;

use crate::app::{AppState, DetectCommand, DetectEvent, DetectStatus, RecordMode};
use crate::demo::{demo_default_path, demo_playback_loop, resolve_demo_path, reset_store_for_demo};
use crate::recording::{record_status_snapshot, stop_recording_internal, RecordStatusResponse};
use crate::utils::{hex_encode, now_epoch_ms};
use crate::ws::ws_handler;

mod types;
use types::*;

pub fn router(app_state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/config/udp", get(get_udp_config).post(set_udp_config))
        .route("/config/udp/auto-detect", axum::routing::post(start_auto_detect))
        .route(
            "/config/udp/auto-detect/cancel",
            axum::routing::post(cancel_auto_detect),
        )
        .route("/config/udp/auto-detect/:id", get(get_auto_detect_status))
        .route("/demo/status", get(get_demo_status))
        .route("/demo/start", axum::routing::post(start_demo_playback))
        .route("/demo/stop", axum::routing::post(stop_demo_playback))
        .route("/demo/record/status", get(get_record_status))
        .route("/demo/record/start", axum::routing::post(start_recording))
        .route("/demo/record/stop", axum::routing::post(stop_recording))
        .route("/meta/car/:id", get(get_meta_car))
        .route("/meta/track/:id", get(get_meta_track))
        .route("/meta/track/:id/geometry", get(get_meta_track_geometry))
        .route("/meta/track/:id/geometry/svg", get(get_meta_track_geometry_svg))
        .route("/meta/current", get(get_meta_current))
        .route("/debug/telemetry", get(get_debug_telemetry))
        .route("/ws", get(ws_handler))
        .with_state(app_state)
}

async fn health() -> impl IntoResponse {
    Json(HealthResponse { status: "ok" })
}

async fn get_udp_config(AxumState(app_state): AxumState<AppState>) -> impl IntoResponse {
    let config = app_state.udp_config_tx.borrow().clone();
    Json(config)
}

async fn set_udp_config(
    AxumState(app_state): AxumState<AppState>,
    Json(payload): Json<crate::app::UdpConfig>,
) -> impl IntoResponse {
    if payload.ps5_ip.is_some() {
        let mut store = app_state.detect_store.write().await;
        if let Some(active_id) = store.active_id {
            if let Some(session) = store.sessions.get_mut(&active_id) {
                session.status = DetectStatus::Cancelled;
                session.ps5_ip = None;
                store.last_event = Some(DetectEvent {
                    id: active_id,
                    status: DetectStatus::Cancelled,
                });
                info!(id = active_id, "manual ps5_ip override cancelled auto-detect");
            }
            store.active_id = None;
        }
    }
    let _ = app_state.udp_config_tx.send(payload.clone());
    Json(payload)
}

async fn start_auto_detect(AxumState(app_state): AxumState<AppState>) -> impl IntoResponse {
    let mut store = app_state.detect_store.write().await;
    if let Some(active_id) = store.active_id {
        if let Some(session) = store.sessions.get(&active_id) {
            info!(id = active_id, "auto-detect already active");
            return Json(DetectStartResponse {
                id: session.id,
                status: session.status.clone(),
                timeout_ms: session.timeout_ms,
            });
        }
    }

    let id = crate::utils::next_sequence(app_state.detect_sequence.as_ref());
    let timeout_ms = 10_000;
    let session = crate::app::DetectSession {
        id,
        status: DetectStatus::Pending,
        ps5_ip: None,
        timeout_ms,
    };
    store.sessions.insert(id, session.clone());
    store.active_id = Some(id);
    store.last_event = Some(DetectEvent {
        id,
        status: DetectStatus::Pending,
    });
    drop(store);

    info!(id, "auto-detect session started");
    let _ = app_state
        .detect_tx
        .send(DetectCommand { id, timeout_ms })
        .await;

    Json(DetectStartResponse {
        id,
        status: DetectStatus::Pending,
        timeout_ms,
    })
}

async fn cancel_auto_detect(AxumState(app_state): AxumState<AppState>) -> impl IntoResponse {
    let mut store = app_state.detect_store.write().await;
    if let Some(active_id) = store.active_id {
        if let Some(session) = store.sessions.get_mut(&active_id) {
            session.status = DetectStatus::Cancelled;
            session.ps5_ip = None;
            store.last_event = Some(DetectEvent {
                id: active_id,
                status: DetectStatus::Cancelled,
            });
        }
        store.active_id = None;
        info!(id = active_id, "auto-detect session cancelled");
        return Json(json!({
            "status": "cancelled",
            "id": active_id,
        }));
    }

    info!("auto-detect cancel requested with no active session");
    Json(json!({
        "status": "no_active_session",
        "id": null,
    }))
}

async fn get_auto_detect_status(
    AxumState(app_state): AxumState<AppState>,
    axum::extract::Path(id): axum::extract::Path<u64>,
) -> impl IntoResponse {
    let store = app_state.detect_store.read().await;
    if let Some(session) = store.sessions.get(&id) {
        return Json(DetectStatusResponse {
            id: session.id,
            status: session.status.clone(),
            ps5_ip: session.ps5_ip,
        });
    }

    Json(DetectStatusResponse {
        id,
        status: DetectStatus::Error,
        ps5_ip: None,
    })
}

async fn get_demo_status(AxumState(app_state): AxumState<AppState>) -> impl IntoResponse {
    let state = app_state.demo_state.lock().await;
    Json(DemoStatusResponse {
        active: state.active,
        path: state
            .path
            .as_ref()
            .map(|path| path.to_string_lossy().to_string()),
    })
}

async fn start_demo_playback(
    AxumState(app_state): AxumState<AppState>,
) -> Result<Json<DemoStatusResponse>, (StatusCode, Json<serde_json::Value>)> {
    let path = resolve_demo_path(&app_state.data_dir);
    if !path.is_file() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "demo file not found",
                "path": path.to_string_lossy(),
            })),
        ));
    }
    let path_display = path.to_string_lossy().to_string();

    let mut state = app_state.demo_state.lock().await;
    if state.active {
        return Ok(Json(DemoStatusResponse {
            active: true,
            path: state
                .path
                .as_ref()
                .map(|path| path.to_string_lossy().to_string()),
        }));
    }

    let (cancel_tx, cancel_rx) = tokio::sync::oneshot::channel();
    state.active = true;
    state.path = Some(path.clone());
    state.cancel = Some(cancel_tx);
    drop(state);

    app_state.demo_active.store(true, Ordering::Relaxed);
    reset_store_for_demo(&app_state.store).await;

    let store = app_state.store.clone();
    let meta = app_state.meta.clone();
    let demo_state = app_state.demo_state.clone();
    let demo_active = app_state.demo_active.clone();
    let start_instant = app_state.start_instant;

    let playback_path = path.clone();
    tokio::spawn(async move {
        if let Err(err) = demo_playback_loop(playback_path, store, meta, start_instant, cancel_rx).await {
            tracing::warn!(?err, "demo playback failed");
        }
        demo_active.store(false, Ordering::Relaxed);
        let mut state = demo_state.lock().await;
        state.active = false;
        state.cancel = None;
    });

    Ok(Json(DemoStatusResponse {
        active: true,
        path: Some(path_display),
    }))
}

async fn stop_demo_playback(AxumState(app_state): AxumState<AppState>) -> impl IntoResponse {
    let mut state = app_state.demo_state.lock().await;
    if let Some(cancel) = state.cancel.take() {
        let _ = cancel.send(());
    }
    state.active = false;
    app_state.demo_active.store(false, Ordering::Relaxed);
    Json(DemoStatusResponse {
        active: false,
        path: state
            .path
            .as_ref()
            .map(|path| path.to_string_lossy().to_string()),
    })
}

async fn get_record_status(AxumState(app_state): AxumState<AppState>) -> impl IntoResponse {
    let state = app_state.record_state.lock().await;
    Json(record_status_snapshot(&state))
}

async fn start_recording(
    AxumState(app_state): AxumState<AppState>,
) -> Result<Json<RecordStatusResponse>, (StatusCode, Json<serde_json::Value>)> {
    let path = demo_default_path(&app_state.data_dir);
    if let Some(parent) = path.parent() {
        if let Err(err) = tokio::fs::create_dir_all(parent).await {
            tracing::warn!(?err, "failed to create demo directory");
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "failed to create demo directory" })),
            ));
        }
    }

    let mut state = app_state.record_state.lock().await;
    if state.mode != RecordMode::Idle {
        return Ok(Json(record_status_snapshot(&state)));
    }
    state.mode = RecordMode::Armed;
    state.path = Some(path.clone());
    state.start_ms = None;
    state.writer = None;
    state.frames = 0;

    Ok(Json(record_status_snapshot(&state)))
}

async fn stop_recording(AxumState(app_state): AxumState<AppState>) -> impl IntoResponse {
    let response = stop_recording_internal(&app_state.record_state).await;
    Json(response)
}

async fn get_meta_car(
    AxumState(app_state): AxumState<AppState>,
    axum::extract::Path(id): axum::extract::Path<i32>,
) -> impl IntoResponse {
    let info = app_state.meta.get_car_info(id);
    let (name, manufacturer) = match info {
        Some(info) => (Some(info.name.clone()), info.manufacturer.clone()),
        None => (None, None),
    };
    Json(MetaCarResponse {
        id,
        name,
        manufacturer,
    })
}

async fn get_meta_track(
    AxumState(app_state): AxumState<AppState>,
    axum::extract::Path(id): axum::extract::Path<i32>,
) -> impl IntoResponse {
    let info = app_state.meta.get_track_info(id);
    let (name, base_id, layout_number, is_reverse) = match info {
        Some(info) => (
            Some(info.name.clone()),
            info.base_id,
            info.layout_number,
            info.is_reverse,
        ),
        None => (None, None, None, None),
    };
    Json(MetaTrackResponse {
        id,
        name,
        base_id,
        layout_number,
        is_reverse,
    })
}

async fn get_meta_track_geometry(
    AxumState(app_state): AxumState<AppState>,
    axum::extract::Path(id): axum::extract::Path<i32>,
) -> impl IntoResponse {
    let path = app_state.meta.get_track_geometry_path(id);
    Json(MetaTrackGeometryResponse {
        id,
        has_geometry: path.is_some(),
        path: path.map(|path| path.to_string_lossy().to_string()),
    })
}

async fn get_meta_track_geometry_svg(
    AxumState(app_state): AxumState<AppState>,
    axum::extract::Path(id): axum::extract::Path<i32>,
) -> impl IntoResponse {
    let svg = app_state.meta.get_track_geometry_svg(id);
    if let Some(svg) = svg {
        Json(MetaTrackGeometrySvgResponse {
            id,
            exists: true,
            view_box: Some(svg.view_box),
            path_d: Some(svg.path_d),
            points_count: Some(svg.points_count),
            simplified: Some(svg.simplified),
        })
    } else {
        Json(MetaTrackGeometrySvgResponse {
            id,
            exists: false,
            view_box: None,
            path_d: None,
            points_count: None,
            simplified: None,
        })
    }
}

async fn get_meta_current(AxumState(app_state): AxumState<AppState>) -> impl IntoResponse {
    let (car_id, track_id) = {
        let store = app_state.store.read().await;
        (store.session.car_id, store.session.track_id)
    };

    let (car_name, car_manufacturer) = car_id
        .and_then(|id| app_state.meta.get_car_info(id))
        .map(|info| (Some(info.name.clone()), info.manufacturer.clone()))
        .unwrap_or((None, None));

    let track_name = track_id
        .and_then(|id| app_state.meta.get_track_name(id))
        .map(str::to_string);
    let has_geometry = track_id
        .map(|id| app_state.meta.has_track_geometry(id))
        .unwrap_or(false);

    Json(MetaCurrentResponse {
        car_id,
        car_name,
        car_manufacturer,
        track_id,
        track_name,
        has_geometry,
    })
}

async fn get_debug_telemetry(AxumState(app_state): AxumState<AppState>) -> impl IntoResponse {
    let store = app_state.store.read().await;
    let state = store.session.state.clone();
    let raw_latest = store.raw_packets.back();
    let (raw_encrypted_len, raw_decrypted_len, raw_encrypted_hex, raw_decrypted_hex, raw_ip, raw_captured_at) =
        match raw_latest {
            Some(raw) => (
                Some(raw.encrypted.len()),
                Some(raw.decrypted.len()),
                Some(hex_encode(&raw.encrypted)),
                Some(hex_encode(&raw.decrypted)),
                raw.source_ip,
                Some(raw.captured_at_ms),
            ),
            None => (None, None, None, None, None, None),
        };
    Json(DebugTelemetryResponse {
        timestamp_ms: now_epoch_ms(),
        session: DebugSession {
            in_race: state.in_race,
            is_paused: state.is_paused,
            packet_id: state.packet_id,
            time_on_track_ms: state.time_on_track_ms,
            current_lap: state.current_lap,
            total_laps: state.total_laps,
            best_lap_ms: state.best_lap_ms,
            last_lap_ms: state.last_lap_ms,
            current_position: state.current_position,
            total_positions: state.total_positions,
            car_id: store.session.car_id,
            track_id: store.session.track_id,
        },
        powertrain: DebugPowertrain {
            speed_kph: state.speed_kph,
            rpm: state.rpm,
            rpm_rev_warning: state.rpm_rev_warning,
            rpm_rev_limiter: state.rpm_rev_limiter,
            gear: state.gear,
            gear_raw: state.gear_raw,
            suggested_gear: state.suggested_gear,
            throttle: state.throttle,
            brake: state.brake,
            clutch: state.clutch,
            clutch_engaged: state.clutch_engaged,
            rpm_after_clutch: state.rpm_after_clutch,
            boost_kpa: state.boost_kpa,
            estimated_speed_kph: state.estimated_speed_kph,
        },
        fluids: DebugFluids {
            fuel_l: state.fuel_l,
            fuel_capacity_l: state.fuel_capacity_l,
            oil_temp_c: state.oil_temp_c,
            water_temp_c: state.water_temp_c,
            oil_pressure_kpa: state.oil_pressure_kpa,
        },
        tyres: DebugTyres {
            temp_fl_c: state.temp_fl_c,
            temp_fr_c: state.temp_fr_c,
            temp_rl_c: state.temp_rl_c,
            temp_rr_c: state.temp_rr_c,
            tyre_diameter_fl_m: state.tyre_diameter_fl_m,
            tyre_diameter_fr_m: state.tyre_diameter_fr_m,
            tyre_diameter_rl_m: state.tyre_diameter_rl_m,
            tyre_diameter_rr_m: state.tyre_diameter_rr_m,
        },
        wheels: DebugWheels {
            wheel_speed_fl: state.wheel_speed_fl,
            wheel_speed_fr: state.wheel_speed_fr,
            wheel_speed_rl: state.wheel_speed_rl,
            wheel_speed_rr: state.wheel_speed_rr,
            tyre_speed_fl_kph: state.tyre_speed_fl_kph,
            tyre_speed_fr_kph: state.tyre_speed_fr_kph,
            tyre_speed_rl_kph: state.tyre_speed_rl_kph,
            tyre_speed_rr_kph: state.tyre_speed_rr_kph,
            tyre_slip_ratio_fl: state.tyre_slip_ratio_fl,
            tyre_slip_ratio_fr: state.tyre_slip_ratio_fr,
            tyre_slip_ratio_rl: state.tyre_slip_ratio_rl,
            tyre_slip_ratio_rr: state.tyre_slip_ratio_rr,
        },
        chassis: DebugChassis {
            ride_height_mm: state.ride_height_mm,
            suspension_fl: state.suspension_fl,
            suspension_fr: state.suspension_fr,
            suspension_rl: state.suspension_rl,
            suspension_rr: state.suspension_rr,
        },
        gears: DebugGears {
            gear_ratio_1: state.gear_ratio_1,
            gear_ratio_2: state.gear_ratio_2,
            gear_ratio_3: state.gear_ratio_3,
            gear_ratio_4: state.gear_ratio_4,
            gear_ratio_5: state.gear_ratio_5,
            gear_ratio_6: state.gear_ratio_6,
            gear_ratio_7: state.gear_ratio_7,
            gear_ratio_8: state.gear_ratio_8,
            gear_ratio_unknown: state.gear_ratio_unknown,
        },
        dynamics: DebugDynamics {
            pos_x: state.pos_x,
            pos_y: state.pos_y,
            pos_z: state.pos_z,
            vel_x: state.vel_x,
            vel_y: state.vel_y,
            vel_z: state.vel_z,
            angular_vel_x: state.angular_vel_x,
            angular_vel_y: state.angular_vel_y,
            angular_vel_z: state.angular_vel_z,
            accel_long: None,
            accel_lat: None,
            yaw_rate: state.yaw_rate,
            pitch: state.pitch,
            roll: state.roll,
            rotation_yaw: state.rotation_yaw,
            rotation_extra: state.rotation_extra,
        },
        flags: DebugFlags {
            flags_8e: state.flags_8e,
            flags_8f: state.flags_8f,
            flags_93: state.flags_93,
            unknown_0x94: state.unknown_0x94,
            unknown_0x98: state.unknown_0x98,
            unknown_0x9c: state.unknown_0x9c,
            unknown_0xa0: state.unknown_0xa0,
            unknown_0xd4: state.unknown_0xd4,
            unknown_0xd8: state.unknown_0xd8,
            unknown_0xdc: state.unknown_0xdc,
            unknown_0xe0: state.unknown_0xe0,
            unknown_0xe4: state.unknown_0xe4,
            unknown_0xe8: state.unknown_0xe8,
            unknown_0xec: state.unknown_0xec,
            unknown_0xf0: state.unknown_0xf0,
        },
        raw: DebugRaw {
            encrypted_len: raw_encrypted_len.or(store.last_packet_len),
            decrypted_len: raw_decrypted_len.or(store.last_payload_len),
            encrypted_hex: raw_encrypted_hex,
            decrypted_hex: raw_decrypted_hex,
            source_ip: raw_ip.or(store.last_source_ip),
            captured_at_ms: raw_captured_at,
            last_telemetry_ms: store.last_telemetry_ms,
            last_source_timestamp_ms: store.last_source_timestamp_ms,
        },
    })
}
