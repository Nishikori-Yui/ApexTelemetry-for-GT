export type MetaCarResponse = {
  id: number
  name?: string | null
  manufacturer?: string | null
}

export type MetaTrackResponse = {
  id: number
  name?: string | null
  base_id?: number | null
  layout_number?: number | null
  is_reverse?: boolean | null
}

export type DebugTelemetryResponse = {
  timestamp_ms: number
  session: {
    in_race?: boolean | null
    is_paused?: boolean | null
    packet_id?: number | null
    time_on_track_ms?: number | null
    current_lap?: number | null
    total_laps?: number | null
    best_lap_ms?: number | null
    last_lap_ms?: number | null
    current_position?: number | null
    total_positions?: number | null
    car_id?: number | null
    track_id?: number | null
  }
  powertrain: {
    speed_kph?: number | null
    rpm?: number | null
    rpm_rev_warning?: number | null
    rpm_rev_limiter?: number | null
    gear?: number | null
    gear_raw?: number | null
    suggested_gear?: number | null
    throttle?: number | null
    brake?: number | null
    clutch?: number | null
    clutch_engaged?: number | null
    rpm_after_clutch?: number | null
    boost_kpa?: number | null
    estimated_speed_kph?: number | null
  }
  fluids: {
    fuel_l?: number | null
    fuel_capacity_l?: number | null
    oil_temp_c?: number | null
    water_temp_c?: number | null
    oil_pressure_kpa?: number | null
  }
  tyres: {
    temp_fl_c?: number | null
    temp_fr_c?: number | null
    temp_rl_c?: number | null
    temp_rr_c?: number | null
    tyre_diameter_fl_m?: number | null
    tyre_diameter_fr_m?: number | null
    tyre_diameter_rl_m?: number | null
    tyre_diameter_rr_m?: number | null
  }
  wheels: {
    wheel_speed_fl?: number | null
    wheel_speed_fr?: number | null
    wheel_speed_rl?: number | null
    wheel_speed_rr?: number | null
    tyre_speed_fl_kph?: number | null
    tyre_speed_fr_kph?: number | null
    tyre_speed_rl_kph?: number | null
    tyre_speed_rr_kph?: number | null
    tyre_slip_ratio_fl?: number | null
    tyre_slip_ratio_fr?: number | null
    tyre_slip_ratio_rl?: number | null
    tyre_slip_ratio_rr?: number | null
  }
  chassis: {
    ride_height_mm?: number | null
    suspension_fl?: number | null
    suspension_fr?: number | null
    suspension_rl?: number | null
    suspension_rr?: number | null
  }
  gears: {
    gear_ratio_1?: number | null
    gear_ratio_2?: number | null
    gear_ratio_3?: number | null
    gear_ratio_4?: number | null
    gear_ratio_5?: number | null
    gear_ratio_6?: number | null
    gear_ratio_7?: number | null
    gear_ratio_8?: number | null
    gear_ratio_unknown?: number | null
  }
  dynamics: {
    pos_x?: number | null
    pos_y?: number | null
    pos_z?: number | null
    vel_x?: number | null
    vel_y?: number | null
    vel_z?: number | null
    angular_vel_x?: number | null
    angular_vel_y?: number | null
    angular_vel_z?: number | null
    accel_long?: number | null
    accel_lat?: number | null
    yaw_rate?: number | null
    pitch?: number | null
    roll?: number | null
    rotation_yaw?: number | null
    rotation_extra?: number | null
  }
  flags: {
    flags_8e?: number | null
    flags_8f?: number | null
    flags_93?: number | null
    unknown_0x94?: number | null
    unknown_0x98?: number | null
    unknown_0x9c?: number | null
    unknown_0xa0?: number | null
    unknown_0xd4?: number | null
    unknown_0xd8?: number | null
    unknown_0xdc?: number | null
    unknown_0xe0?: number | null
    unknown_0xe4?: number | null
    unknown_0xe8?: number | null
    unknown_0xec?: number | null
    unknown_0xf0?: number | null
  }
  raw: {
    encrypted_len?: number | null
    decrypted_len?: number | null
    encrypted_hex?: string | null
    decrypted_hex?: string | null
    source_ip?: string | null
    captured_at_ms?: number | null
    last_telemetry_ms?: number | null
    last_source_timestamp_ms?: number | null
  }
}

export type TrackGeometrySvg = {
  id: number
  exists: boolean
  view_box?: string | null
  path_d?: string | null
  points_count?: number | null
  simplified?: boolean | null
}

export type UdpConfig = {
  bind_addr: string
  ps5_ip: string | null
}

export type DetectStatus =
  | 'idle'
  | 'pending'
  | 'found'
  | 'timeout'
  | 'error'
  | 'cancelled'

export type DetectStartResponse = {
  id: number
  status: DetectStatus
  timeout_ms: number
}

export type DetectStatusResponse = {
  id: number
  status: DetectStatus
  ps5_ip: string | null
}

export type DemoStatusResponse = {
  active: boolean
  path?: string | null
}
