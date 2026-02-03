export type TelemetryState = {
  speed_kph?: number
  rpm?: number
  gear?: number
  throttle?: number
  brake?: number
  boost_kpa?: number
  in_race?: boolean
  is_paused?: boolean
  packet_id?: number
  current_position?: number
  total_positions?: number
  current_lap?: number
  total_laps?: number
  best_lap_ms?: number
  last_lap_ms?: number
  time_on_track_ms?: number
  current_lap_time_ms?: number
  fuel_l?: number
  fuel_capacity_l?: number
  car_id?: number
  track_id?: number
  avg_fuel_consume_pct_per_lap?: number
  fuel_laps_remaining?: number
  pos_x?: number
  pos_y?: number
  pos_z?: number
  vel_x?: number
  vel_y?: number
  vel_z?: number
  rotation_yaw?: number
}

export type Sample = {
  t_ms: number
  speed_kph?: number
  rpm?: number
  throttle?: number
  brake?: number
}
