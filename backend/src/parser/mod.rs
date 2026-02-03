// GT7 telemetry parser.
// Invariants: uncertain fields are represented as Option or Unknown; no guessing.

use crate::model::TelemetryFrame;

pub fn parse_telemetry(payload: &[u8]) -> Option<TelemetryFrame> {
    let rpm = read_f32(payload, 0x3C);
    let rpm_rev_warning = read_u16(payload, 0x88);
    let rpm_rev_limiter = read_u16(payload, 0x8A);
    let speed_ms = read_f32(payload, 0x4C);
    let speed_kph = speed_ms.map(|value| value * 3.6);
    let fuel_l = read_f32(payload, 0x44);
    let fuel_capacity_l = read_f32(payload, 0x48);
    let boost_bar = read_f32(payload, 0x50).map(|value| value - 1.0);
    let boost_kpa = boost_bar.map(|value| value * 100.0);
    let oil_pressure_bar = read_f32(payload, 0x54);
    let oil_pressure_kpa = oil_pressure_bar.map(|value| value * 100.0);
    let water_temp_c = read_f32(payload, 0x58);
    let oil_temp_c = read_f32(payload, 0x5C);
    let temp_fl_c = read_f32(payload, 0x60);
    let temp_fr_c = read_f32(payload, 0x64);
    let temp_rl_c = read_f32(payload, 0x68);
    let temp_rr_c = read_f32(payload, 0x6C);
    let tyre_diameter_fl_m = read_f32(payload, 0xB4);
    let tyre_diameter_fr_m = read_f32(payload, 0xB8);
    let tyre_diameter_rl_m = read_f32(payload, 0xBC);
    let tyre_diameter_rr_m = read_f32(payload, 0xC0);
    let wheel_speed_fl = read_f32(payload, 0xA4);
    let wheel_speed_fr = read_f32(payload, 0xA8);
    let wheel_speed_rl = read_f32(payload, 0xAC);
    let wheel_speed_rr = read_f32(payload, 0xB0);
    let tyre_speed_fl_kph = match (tyre_diameter_fl_m, wheel_speed_fl) {
        (Some(diameter), Some(wheel_speed)) => Some((diameter * wheel_speed * 3.6).abs()),
        _ => None,
    };
    let tyre_speed_fr_kph = match (tyre_diameter_fr_m, wheel_speed_fr) {
        (Some(diameter), Some(wheel_speed)) => Some((diameter * wheel_speed * 3.6).abs()),
        _ => None,
    };
    let tyre_speed_rl_kph = match (tyre_diameter_rl_m, wheel_speed_rl) {
        (Some(diameter), Some(wheel_speed)) => Some((diameter * wheel_speed * 3.6).abs()),
        _ => None,
    };
    let tyre_speed_rr_kph = match (tyre_diameter_rr_m, wheel_speed_rr) {
        (Some(diameter), Some(wheel_speed)) => Some((diameter * wheel_speed * 3.6).abs()),
        _ => None,
    };
    let tyre_slip_ratio_fl = match (tyre_speed_fl_kph, speed_kph) {
        (Some(tyre_speed), Some(car_speed)) if car_speed > 0.0 => Some(tyre_speed / car_speed),
        _ => None,
    };
    let tyre_slip_ratio_fr = match (tyre_speed_fr_kph, speed_kph) {
        (Some(tyre_speed), Some(car_speed)) if car_speed > 0.0 => Some(tyre_speed / car_speed),
        _ => None,
    };
    let tyre_slip_ratio_rl = match (tyre_speed_rl_kph, speed_kph) {
        (Some(tyre_speed), Some(car_speed)) if car_speed > 0.0 => Some(tyre_speed / car_speed),
        _ => None,
    };
    let tyre_slip_ratio_rr = match (tyre_speed_rr_kph, speed_kph) {
        (Some(tyre_speed), Some(car_speed)) if car_speed > 0.0 => Some(tyre_speed / car_speed),
        _ => None,
    };
    let suspension_fl = read_f32(payload, 0xC4);
    let suspension_fr = read_f32(payload, 0xC8);
    let suspension_rl = read_f32(payload, 0xCC);
    let suspension_rr = read_f32(payload, 0xD0);
    let gear_ratio_1 = read_f32(payload, 0x104);
    let gear_ratio_2 = read_f32(payload, 0x108);
    let gear_ratio_3 = read_f32(payload, 0x10C);
    let gear_ratio_4 = read_f32(payload, 0x110);
    let gear_ratio_5 = read_f32(payload, 0x114);
    let gear_ratio_6 = read_f32(payload, 0x118);
    let gear_ratio_7 = read_f32(payload, 0x11C);
    let gear_ratio_8 = read_f32(payload, 0x120);
    let gear_ratio_unknown = read_f32(payload, 0x100);
    let pos_x = read_f32(payload, 0x04);
    let pos_y = read_f32(payload, 0x08);
    let pos_z = read_f32(payload, 0x0C);
    let vel_x = read_f32(payload, 0x10);
    let vel_y = read_f32(payload, 0x14);
    let vel_z = read_f32(payload, 0x18);
    let pitch = read_f32(payload, 0x1C);
    let rotation_yaw = read_f32(payload, 0x20); // Yaw likely at 0x20
    let roll = read_f32(payload, 0x24);
    let rotation_extra = read_f32(payload, 0x28);
    let angular_vel_x = read_f32(payload, 0x2C);
    let angular_vel_y = read_f32(payload, 0x30);
    let angular_vel_z = read_f32(payload, 0x34);
    let yaw_rate = angular_vel_y;
    let ride_height_mm = read_f32(payload, 0x38).map(|value| value * 1000.0);
    let throttle = read_u8(payload, 0x91).map(|value| value as f32 / 255.0);
    let brake = read_u8(payload, 0x92).map(|value| value as f32 / 255.0);
    let clutch = read_f32(payload, 0xF4);
    let clutch_engaged = read_f32(payload, 0xF8);
    let rpm_after_clutch = read_f32(payload, 0xFC);
    let gear_byte = read_u8(payload, 0x90);
    let gear_raw = gear_byte.map(|value| value & 0x0F);
    let suggested_gear = gear_byte.map(|value| value >> 4);
    let gear = gear_raw.map(|value| if value == 0 { -1 } else { value as i8 });
    let estimated_speed_kph = read_i16(payload, 0x8C).map(|value| value as f32);
    let packet_id = read_i32(payload, 0x70);
    let current_lap = read_i16(payload, 0x74);
    let total_laps = read_i16(payload, 0x76);
    let best_lap_ms = read_i32(payload, 0x78);
    let last_lap_ms = read_i32(payload, 0x7C);
    let time_on_track_ms = read_i32(payload, 0x80);
    let current_position = read_i16(payload, 0x84);
    let total_positions = read_i16(payload, 0x86);
    let flags_8e = read_u8(payload, 0x8E);
    let flags_8f = read_u8(payload, 0x8F);
    let flags_93 = read_u8(payload, 0x93);
    let in_race = flags_8e.map(|value| (value & 0b0000_0001) != 0);
    let is_paused = flags_8e.map(|value| (value & 0b0000_0010) != 0);
    let unknown_0x94 = read_f32(payload, 0x94);
    let unknown_0x98 = read_f32(payload, 0x98);
    let unknown_0x9c = read_f32(payload, 0x9C);
    let unknown_0xa0 = read_f32(payload, 0xA0);
    let unknown_0xd4 = read_f32(payload, 0xD4);
    let unknown_0xd8 = read_f32(payload, 0xD8);
    let unknown_0xdc = read_f32(payload, 0xDC);
    let unknown_0xe0 = read_f32(payload, 0xE0);
    let unknown_0xe4 = read_f32(payload, 0xE4);
    let unknown_0xe8 = read_f32(payload, 0xE8);
    let unknown_0xec = read_f32(payload, 0xEC);
    let unknown_0xf0 = read_f32(payload, 0xF0);
    let car_id = read_i32(payload, 0x124);

    let has_any = rpm.is_some()
        || rpm_rev_warning.is_some()
        || rpm_rev_limiter.is_some()
        || speed_kph.is_some()
        || throttle.is_some()
        || brake.is_some()
        || clutch.is_some()
        || clutch_engaged.is_some()
        || rpm_after_clutch.is_some()
        || boost_kpa.is_some()
        || estimated_speed_kph.is_some()
        || fuel_l.is_some()
        || fuel_capacity_l.is_some()
        || oil_temp_c.is_some()
        || water_temp_c.is_some()
        || oil_pressure_kpa.is_some()
        || ride_height_mm.is_some()
        || temp_fl_c.is_some()
        || temp_fr_c.is_some()
        || temp_rl_c.is_some()
        || temp_rr_c.is_some()
        || tyre_diameter_fl_m.is_some()
        || tyre_diameter_fr_m.is_some()
        || tyre_diameter_rl_m.is_some()
        || tyre_diameter_rr_m.is_some()
        || wheel_speed_fl.is_some()
        || wheel_speed_fr.is_some()
        || wheel_speed_rl.is_some()
        || wheel_speed_rr.is_some()
        || tyre_speed_fl_kph.is_some()
        || tyre_speed_fr_kph.is_some()
        || tyre_speed_rl_kph.is_some()
        || tyre_speed_rr_kph.is_some()
        || tyre_slip_ratio_fl.is_some()
        || tyre_slip_ratio_fr.is_some()
        || tyre_slip_ratio_rl.is_some()
        || tyre_slip_ratio_rr.is_some()
        || suspension_fl.is_some()
        || suspension_fr.is_some()
        || suspension_rl.is_some()
        || suspension_rr.is_some()
        || gear_ratio_1.is_some()
        || gear_ratio_2.is_some()
        || gear_ratio_3.is_some()
        || gear_ratio_4.is_some()
        || gear_ratio_5.is_some()
        || gear_ratio_6.is_some()
        || gear_ratio_7.is_some()
        || gear_ratio_8.is_some()
        || gear_ratio_unknown.is_some()
        || vel_x.is_some()
        || vel_y.is_some()
        || vel_z.is_some()
        || angular_vel_x.is_some()
        || angular_vel_y.is_some()
        || angular_vel_z.is_some()
        || yaw_rate.is_some()
        || pitch.is_some()
        || roll.is_some()
        || gear.is_some()
        || gear_raw.is_some()
        || suggested_gear.is_some()
        || packet_id.is_some()
        || current_position.is_some()
        || total_positions.is_some()
        || current_lap.is_some()
        || total_laps.is_some()
        || best_lap_ms.is_some()
        || last_lap_ms.is_some()
        || time_on_track_ms.is_some()
        || car_id.is_some()
        || pos_x.is_some()
        || pos_y.is_some()
        || pos_z.is_some()
        || rotation_yaw.is_some()
        || rotation_extra.is_some()
        || flags_8e.is_some()
        || flags_8f.is_some()
        || flags_93.is_some()
        || unknown_0x94.is_some()
        || unknown_0x98.is_some()
        || unknown_0x9c.is_some()
        || unknown_0xa0.is_some()
        || unknown_0xd4.is_some()
        || unknown_0xd8.is_some()
        || unknown_0xdc.is_some()
        || unknown_0xe0.is_some()
        || unknown_0xe4.is_some()
        || unknown_0xe8.is_some()
        || unknown_0xec.is_some()
        || unknown_0xf0.is_some()
        || in_race.is_some()
        || is_paused.is_some();

    if !has_any {
        return None;
    }

    Some(TelemetryFrame {
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
        car_id,
        track_id: None,
        source_timestamp_ms: None,
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

fn read_u16(payload: &[u8], offset: usize) -> Option<u16> {
    let bytes = payload.get(offset..offset + 2)?;
    Some(u16::from_le_bytes(bytes.try_into().ok()?))
}

fn read_i32(payload: &[u8], offset: usize) -> Option<i32> {
    let bytes = payload.get(offset..offset + 4)?;
    Some(i32::from_le_bytes(bytes.try_into().ok()?))
}
