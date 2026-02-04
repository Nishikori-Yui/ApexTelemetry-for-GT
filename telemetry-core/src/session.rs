// Session state tracking and derived telemetry fields.

use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

use crate::model::{State, TelemetryFrame};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum SessionState {
    NotInRace,
    InRace,
    Paused,
}

#[derive(Clone, Copy, Debug)]
pub struct SessionTransition {
    pub from: SessionState,
    pub to: SessionState,
}

#[derive(Clone, Copy, Debug)]
pub struct SessionEvents {
    pub transition: Option<SessionTransition>,
    pub should_stop_record: bool,
    pub should_start_record: bool,
}

pub struct SessionFields<'a> {
    pub state: &'a mut State,
    pub session_state: &'a mut SessionState,
    pub session_index: &'a mut u64,
    pub last_current_lap: &'a mut Option<i16>,
    pub last_lap_time_ms_recorded: &'a mut Option<i32>,
    pub lap_start_mono_ms: &'a mut Option<u64>,
    pub lap_pause_started_ms: &'a mut Option<u64>,
    pub lap_pause_accum_ms: &'a mut u64,
    pub fuel_pct_at_lap_start: &'a mut Option<f32>,
    pub fuel_consume_history: &'a mut VecDeque<f32>,
    pub car_id: &'a mut Option<i32>,
    pub track_id: &'a mut Option<i32>,
}

impl<'a> SessionFields<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        state: &'a mut State,
        session_state: &'a mut SessionState,
        session_index: &'a mut u64,
        last_current_lap: &'a mut Option<i16>,
        last_lap_time_ms_recorded: &'a mut Option<i32>,
        lap_start_mono_ms: &'a mut Option<u64>,
        lap_pause_started_ms: &'a mut Option<u64>,
        lap_pause_accum_ms: &'a mut u64,
        fuel_pct_at_lap_start: &'a mut Option<f32>,
        fuel_consume_history: &'a mut VecDeque<f32>,
        car_id: &'a mut Option<i32>,
        track_id: &'a mut Option<i32>,
    ) -> Self {
        Self {
            state,
            session_state,
            session_index,
            last_current_lap,
            last_lap_time_ms_recorded,
            lap_start_mono_ms,
            lap_pause_started_ms,
            lap_pause_accum_ms,
            fuel_pct_at_lap_start,
            fuel_consume_history,
            car_id,
            track_id,
        }
    }
}

pub fn next_session_state(current: SessionState, frame: &TelemetryFrame) -> SessionState {
    let (Some(in_race), Some(is_paused)) = (frame.in_race, frame.is_paused) else {
        return current;
    };

    if !in_race {
        SessionState::NotInRace
    } else if is_paused {
        SessionState::Paused
    } else {
        SessionState::InRace
    }
}

pub fn apply_frame(
    fields: &mut SessionFields<'_>,
    frame: &TelemetryFrame,
    now_ms: u64,
    packet_car_id: Option<i32>,
) -> SessionEvents {
    let previous_state = *fields.session_state;
    let next_state = next_session_state(previous_state, frame);
    let should_stop_record = next_state == SessionState::NotInRace;
    let mut transition = None;

    if next_state != previous_state {
        transition = Some(SessionTransition {
            from: previous_state,
            to: next_state,
        });
        if next_state == SessionState::InRace && previous_state == SessionState::NotInRace {
            *fields.session_index = fields.session_index.saturating_add(1);
            *fields.last_current_lap = None;
            *fields.last_lap_time_ms_recorded = None;
            *fields.lap_start_mono_ms = Some(now_ms);
            *fields.lap_pause_started_ms = None;
            *fields.lap_pause_accum_ms = 0;
            *fields.fuel_pct_at_lap_start = None;
            fields.fuel_consume_history.clear();
            *fields.track_id = None;
        } else if next_state == SessionState::NotInRace {
            *fields.track_id = None;
            *fields.lap_start_mono_ms = None;
            *fields.lap_pause_started_ms = None;
            *fields.lap_pause_accum_ms = 0;
            fields.state.current_lap_time_ms = None;
        }
        *fields.session_state = next_state;
    }

    fields.state.update_from(frame);

    if let Some(last_lap_ms) = frame.last_lap_ms {
        if *fields.last_lap_time_ms_recorded != Some(last_lap_ms) {
            *fields.last_lap_time_ms_recorded = Some(last_lap_ms);
            if *fields.session_state != SessionState::NotInRace {
                *fields.lap_start_mono_ms = Some(now_ms);
                *fields.lap_pause_started_ms = None;
                *fields.lap_pause_accum_ms = 0;
            }
        }
    }
    if *fields.session_state == SessionState::InRace && fields.lap_start_mono_ms.is_none() {
        *fields.lap_start_mono_ms = Some(now_ms);
    }

    let should_start_record = *fields.session_state == SessionState::InRace;

    match *fields.session_state {
        SessionState::Paused => {
            if fields.lap_pause_started_ms.is_none() {
                *fields.lap_pause_started_ms = Some(now_ms);
            }
        }
        SessionState::InRace => {
            if let Some(pause_start) = fields.lap_pause_started_ms.take() {
                *fields.lap_pause_accum_ms = fields
                    .lap_pause_accum_ms
                    .saturating_add(now_ms.saturating_sub(pause_start));
            }
        }
        SessionState::NotInRace => {
            *fields.lap_pause_started_ms = None;
            *fields.lap_pause_accum_ms = 0;
        }
    }

    if let Some(lap_start) = *fields.lap_start_mono_ms {
        let mut elapsed = now_ms.saturating_sub(lap_start);
        elapsed = elapsed.saturating_sub(*fields.lap_pause_accum_ms);
        if let Some(pause_start) = *fields.lap_pause_started_ms {
            elapsed = elapsed.saturating_sub(now_ms.saturating_sub(pause_start));
        }
        let safe_elapsed = elapsed.min(i32::MAX as u64) as i32;
        fields.state.current_lap_time_ms = Some(safe_elapsed);
    } else {
        fields.state.current_lap_time_ms = None;
    }

    let current_fuel_pct = match (frame.fuel_l, frame.fuel_capacity_l) {
        (Some(fuel), Some(cap)) if cap > 0.0 => Some((fuel / cap) * 100.0),
        _ => None,
    };

    if let Some(current_lap) = frame.current_lap {
        let lap_changed = fields
            .last_current_lap
            .map(|prev| prev != current_lap)
            .unwrap_or(true);
        if lap_changed && fields.last_current_lap.is_some() {
            let valid_lap = fields
                .last_lap_time_ms_recorded
                .map(|t| t > 0)
                .unwrap_or(false);

            if valid_lap {
                if let (Some(start_pct), Some(end_pct)) =
                    (*fields.fuel_pct_at_lap_start, current_fuel_pct)
                {
                    let consume = (start_pct - end_pct).max(0.0);
                    if consume > 0.0 {
                        if fields.fuel_consume_history.len() >= 3 {
                            fields.fuel_consume_history.pop_back();
                        }
                        fields.fuel_consume_history.push_front(consume);
                    }
                }
            }
        }
        if lap_changed || fields.fuel_pct_at_lap_start.is_none() {
            *fields.fuel_pct_at_lap_start = current_fuel_pct;
        }

        *fields.last_current_lap = Some(current_lap);
    }

    if !fields.fuel_consume_history.is_empty() {
        let sum: f32 = fields.fuel_consume_history.iter().sum();
        let avg = sum / fields.fuel_consume_history.len() as f32;
        fields.state.avg_fuel_consume_pct_per_lap = Some(avg);
        if let Some(fuel_pct) = current_fuel_pct {
            if avg > 0.0 {
                fields.state.fuel_laps_remaining = Some(fuel_pct / avg);
            }
        }
    }

    if let Some(car_id) = packet_car_id {
        *fields.car_id = Some(car_id);
    }
    fields.state.car_id = *fields.car_id;
    fields.state.track_id = *fields.track_id;

    fields.state.pos_x = frame.pos_x;
    fields.state.pos_y = frame.pos_y;
    fields.state.pos_z = frame.pos_z;

    fields.state.vel_x = frame.vel_x;
    fields.state.vel_y = frame.vel_y;
    fields.state.vel_z = frame.vel_z;

    fields.state.rotation_yaw = frame.rotation_yaw;

    SessionEvents {
        transition,
        should_stop_record,
        should_start_record,
    }
}

#[derive(Clone, Debug)]
pub struct SessionTracker {
    pub state: State,
    pub session_state: SessionState,
    pub session_index: u64,
    pub last_current_lap: Option<i16>,
    pub last_lap_time_ms_recorded: Option<i32>,
    pub lap_start_mono_ms: Option<u64>,
    pub lap_pause_started_ms: Option<u64>,
    pub lap_pause_accum_ms: u64,
    pub fuel_pct_at_lap_start: Option<f32>,
    pub fuel_consume_history: VecDeque<f32>,
    pub car_id: Option<i32>,
    pub track_id: Option<i32>,
}

impl SessionTracker {
    pub fn new() -> Self {
        Self {
            state: State::default(),
            session_state: SessionState::NotInRace,
            session_index: 0,
            last_current_lap: None,
            last_lap_time_ms_recorded: None,
            lap_start_mono_ms: None,
            lap_pause_started_ms: None,
            lap_pause_accum_ms: 0,
            fuel_pct_at_lap_start: None,
            fuel_consume_history: VecDeque::with_capacity(3),
            car_id: None,
            track_id: None,
        }
    }

    pub fn reset_for_demo(&mut self) {
        self.state = State::default();
        self.session_state = SessionState::NotInRace;
        self.session_index = self.session_index.saturating_add(1);
        self.last_current_lap = None;
        self.last_lap_time_ms_recorded = None;
        self.lap_start_mono_ms = None;
        self.lap_pause_started_ms = None;
        self.lap_pause_accum_ms = 0;
        self.fuel_pct_at_lap_start = None;
        self.fuel_consume_history.clear();
        self.car_id = None;
        self.track_id = None;
    }

    pub fn apply_frame(
        &mut self,
        frame: &TelemetryFrame,
        now_ms: u64,
        packet_car_id: Option<i32>,
    ) -> SessionEvents {
        let mut fields = SessionFields::new(
            &mut self.state,
            &mut self.session_state,
            &mut self.session_index,
            &mut self.last_current_lap,
            &mut self.last_lap_time_ms_recorded,
            &mut self.lap_start_mono_ms,
            &mut self.lap_pause_started_ms,
            &mut self.lap_pause_accum_ms,
            &mut self.fuel_pct_at_lap_start,
            &mut self.fuel_consume_history,
            &mut self.car_id,
            &mut self.track_id,
        );
        apply_frame(&mut fields, frame, now_ms, packet_car_id)
    }

    pub fn set_track_id(&mut self, track_id: Option<i32>) {
        self.track_id = track_id;
        self.state.track_id = track_id;
    }

    pub fn set_car_id(&mut self, car_id: Option<i32>) {
        self.car_id = car_id;
        self.state.car_id = car_id;
    }
}

impl Default for SessionTracker {
    fn default() -> Self {
        Self::new()
    }
}
