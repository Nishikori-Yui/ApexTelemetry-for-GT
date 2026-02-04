// Telemetry state snapshot used by the UI and streaming layer.

use serde::Serialize;

use super::TelemetryFrame;

#[derive(Clone, Debug, Default, Serialize)]
pub struct State {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed_kph: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rpm: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rpm_rev_warning: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rpm_rev_limiter: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gear: Option<i8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gear_raw: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggested_gear: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub throttle: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brake: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clutch: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clutch_engaged: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rpm_after_clutch: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub boost_kpa: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_speed_kph: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fuel_l: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fuel_capacity_l: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oil_temp_c: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub water_temp_c: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oil_pressure_kpa: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ride_height_mm: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temp_fl_c: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temp_fr_c: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temp_rl_c: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temp_rr_c: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tyre_diameter_fl_m: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tyre_diameter_fr_m: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tyre_diameter_rl_m: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tyre_diameter_rr_m: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wheel_speed_fl: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wheel_speed_fr: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wheel_speed_rl: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wheel_speed_rr: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tyre_speed_fl_kph: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tyre_speed_fr_kph: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tyre_speed_rl_kph: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tyre_speed_rr_kph: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tyre_slip_ratio_fl: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tyre_slip_ratio_fr: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tyre_slip_ratio_rl: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tyre_slip_ratio_rr: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suspension_fl: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suspension_fr: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suspension_rl: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suspension_rr: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gear_ratio_1: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gear_ratio_2: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gear_ratio_3: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gear_ratio_4: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gear_ratio_5: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gear_ratio_6: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gear_ratio_7: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gear_ratio_8: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gear_ratio_unknown: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vel_x: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vel_y: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vel_z: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub angular_vel_x: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub angular_vel_y: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub angular_vel_z: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub yaw_rate: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pitch: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roll: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation_extra: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_race: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_paused: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub packet_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_position: Option<i16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_positions: Option<i16>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_lap_time_ms: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub car_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub track_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_fuel_consume_pct_per_lap: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fuel_laps_remaining: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pos_x: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pos_y: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pos_z: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation_yaw: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flags_8e: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flags_8f: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flags_93: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_0x94: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_0x98: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_0x9c: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_0xa0: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_0xd4: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_0xd8: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_0xdc: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_0xe0: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_0xe4: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_0xe8: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_0xec: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_0xf0: Option<f32>,
}

impl State {
    pub fn update_from(&mut self, frame: &TelemetryFrame) {
        macro_rules! update_field {
            ($field:ident) => {
                if frame.$field.is_some() {
                    self.$field = frame.$field;
                }
            };
        }

        update_field!(speed_kph);
        update_field!(rpm);
        update_field!(rpm_rev_warning);
        update_field!(rpm_rev_limiter);
        update_field!(gear);
        update_field!(gear_raw);
        update_field!(suggested_gear);
        update_field!(throttle);
        update_field!(brake);
        update_field!(clutch);
        update_field!(clutch_engaged);
        update_field!(rpm_after_clutch);
        update_field!(boost_kpa);
        update_field!(estimated_speed_kph);
        update_field!(fuel_l);
        update_field!(fuel_capacity_l);
        update_field!(oil_temp_c);
        update_field!(water_temp_c);
        update_field!(oil_pressure_kpa);
        update_field!(ride_height_mm);
        update_field!(temp_fl_c);
        update_field!(temp_fr_c);
        update_field!(temp_rl_c);
        update_field!(temp_rr_c);
        update_field!(tyre_diameter_fl_m);
        update_field!(tyre_diameter_fr_m);
        update_field!(tyre_diameter_rl_m);
        update_field!(tyre_diameter_rr_m);
        update_field!(wheel_speed_fl);
        update_field!(wheel_speed_fr);
        update_field!(wheel_speed_rl);
        update_field!(wheel_speed_rr);
        update_field!(tyre_speed_fl_kph);
        update_field!(tyre_speed_fr_kph);
        update_field!(tyre_speed_rl_kph);
        update_field!(tyre_speed_rr_kph);
        update_field!(tyre_slip_ratio_fl);
        update_field!(tyre_slip_ratio_fr);
        update_field!(tyre_slip_ratio_rl);
        update_field!(tyre_slip_ratio_rr);
        update_field!(suspension_fl);
        update_field!(suspension_fr);
        update_field!(suspension_rl);
        update_field!(suspension_rr);
        update_field!(gear_ratio_1);
        update_field!(gear_ratio_2);
        update_field!(gear_ratio_3);
        update_field!(gear_ratio_4);
        update_field!(gear_ratio_5);
        update_field!(gear_ratio_6);
        update_field!(gear_ratio_7);
        update_field!(gear_ratio_8);
        update_field!(gear_ratio_unknown);
        update_field!(pos_x);
        update_field!(pos_y);
        update_field!(pos_z);
        update_field!(vel_x);
        update_field!(vel_y);
        update_field!(vel_z);
        update_field!(angular_vel_x);
        update_field!(angular_vel_y);
        update_field!(angular_vel_z);
        update_field!(yaw_rate);
        update_field!(pitch);
        update_field!(roll);
        update_field!(rotation_yaw);
        update_field!(rotation_extra);
        update_field!(in_race);
        update_field!(is_paused);
        update_field!(packet_id);
        update_field!(current_position);
        update_field!(total_positions);
        update_field!(current_lap);
        update_field!(total_laps);
        update_field!(best_lap_ms);
        update_field!(last_lap_ms);
        update_field!(time_on_track_ms);
        update_field!(car_id);
        update_field!(track_id);
        update_field!(flags_8e);
        update_field!(flags_8f);
        update_field!(flags_93);
        update_field!(unknown_0x94);
        update_field!(unknown_0x98);
        update_field!(unknown_0x9c);
        update_field!(unknown_0xa0);
        update_field!(unknown_0xd4);
        update_field!(unknown_0xd8);
        update_field!(unknown_0xdc);
        update_field!(unknown_0xe0);
        update_field!(unknown_0xe4);
        update_field!(unknown_0xe8);
        update_field!(unknown_0xec);
        update_field!(unknown_0xf0);
    }

    pub fn is_empty(&self) -> bool {
        macro_rules! all_none {
            ($($field:ident),+ $(,)?) => {
                $(self.$field.is_none())&&+
            };
        }

        all_none!(
            speed_kph,
            rpm,
            rpm_rev_warning,
            rpm_rev_limiter,
            gear,
            gear_raw,
            suggested_gear,
            throttle,
            brake,
            clutch,
            clutch_engaged,
            rpm_after_clutch,
            boost_kpa,
            estimated_speed_kph,
            fuel_l,
            fuel_capacity_l,
            oil_temp_c,
            water_temp_c,
            oil_pressure_kpa,
            ride_height_mm,
            temp_fl_c,
            temp_fr_c,
            temp_rl_c,
            temp_rr_c,
            tyre_diameter_fl_m,
            tyre_diameter_fr_m,
            tyre_diameter_rl_m,
            tyre_diameter_rr_m,
            wheel_speed_fl,
            wheel_speed_fr,
            wheel_speed_rl,
            wheel_speed_rr,
            tyre_speed_fl_kph,
            tyre_speed_fr_kph,
            tyre_speed_rl_kph,
            tyre_speed_rr_kph,
            tyre_slip_ratio_fl,
            tyre_slip_ratio_fr,
            tyre_slip_ratio_rl,
            tyre_slip_ratio_rr,
            suspension_fl,
            suspension_fr,
            suspension_rl,
            suspension_rr,
            gear_ratio_1,
            gear_ratio_2,
            gear_ratio_3,
            gear_ratio_4,
            gear_ratio_5,
            gear_ratio_6,
            gear_ratio_7,
            gear_ratio_8,
            gear_ratio_unknown,
            pos_x,
            pos_y,
            pos_z,
            vel_x,
            vel_y,
            vel_z,
            angular_vel_x,
            angular_vel_y,
            angular_vel_z,
            yaw_rate,
            pitch,
            roll,
            rotation_yaw,
            rotation_extra,
            in_race,
            is_paused,
            packet_id,
            current_position,
            total_positions,
            current_lap,
            total_laps,
            best_lap_ms,
            last_lap_ms,
            time_on_track_ms,
            current_lap_time_ms,
            car_id,
            track_id,
            avg_fuel_consume_pct_per_lap,
            fuel_laps_remaining,
            flags_8e,
            flags_8f,
            flags_93,
            unknown_0x94,
            unknown_0x98,
            unknown_0x9c,
            unknown_0xa0,
            unknown_0xd4,
            unknown_0xd8,
            unknown_0xdc,
            unknown_0xe0,
            unknown_0xe4,
            unknown_0xe8,
            unknown_0xec,
            unknown_0xf0,
        )
    }
}
