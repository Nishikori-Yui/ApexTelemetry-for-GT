// Track auto-detection based on bounding box overlap.

use std::collections::HashMap;

use super::geometry::TrackBounds;

const TRACK_DETECT_MIN_LAP: i16 = 2;
const TRACK_DETECT_MIN_IOU: f32 = 0.90;

pub struct TrackDetector {
    track_id: Option<i32>,
    min_x: f32,
    max_x: f32,
    min_z: f32,
    max_z: f32,
    has_bounds: bool,
}

impl TrackDetector {
    pub fn new() -> Self {
        let mut detector = Self {
            track_id: None,
            min_x: f32::MAX,
            max_x: f32::MIN,
            min_z: f32::MAX,
            max_z: f32::MIN,
            has_bounds: false,
        };
        detector.reset();
        detector
    }

    pub fn reset(&mut self) {
        self.track_id = None;
        self.min_x = f32::MAX;
        self.max_x = f32::MIN;
        self.min_z = f32::MAX;
        self.max_z = f32::MIN;
        self.has_bounds = false;
    }

    pub fn update(
        &mut self,
        in_race: bool,
        is_paused: bool,
        current_lap: Option<i16>,
        position_xz: Option<(f32, f32)>,
        track_bounds: &HashMap<i32, TrackBounds>,
    ) -> Option<i32> {
        if !in_race || is_paused {
            return self.track_id;
        }

        if let Some((x, z)) = position_xz {
            self.min_x = self.min_x.min(x);
            self.max_x = self.max_x.max(x);
            self.min_z = self.min_z.min(z);
            self.max_z = self.max_z.max(z);
            self.has_bounds = true;
        }

        if self.track_id.is_some() {
            return self.track_id;
        }

        let lap = current_lap.unwrap_or(0);
        if lap < TRACK_DETECT_MIN_LAP {
            return None;
        }

        if !self.has_bounds || track_bounds.is_empty() {
            return None;
        }

        if self.min_x >= self.max_x || self.min_z >= self.max_z {
            return None;
        }

        let mut best_match: Option<(i32, f32)> = None;
        for (track_id, bounds) in track_bounds {
            let iou = bounds_iou(
                self.min_x,
                self.min_z,
                self.max_x,
                self.max_z,
                bounds,
            );
            match best_match {
                Some((_, best_iou)) if iou <= best_iou => {}
                _ => best_match = Some((*track_id, iou)),
            }
        }

        if let Some((track_id, iou)) = best_match {
            if iou >= TRACK_DETECT_MIN_IOU {
                self.track_id = Some(track_id);
            }
        }

        self.track_id
    }
}

fn bounds_iou(min_x: f32, min_z: f32, max_x: f32, max_z: f32, other: &TrackBounds) -> f32 {
    let inter_min_x = min_x.max(other.min_x);
    let inter_max_x = max_x.min(other.max_x);
    let inter_min_z = min_z.max(other.min_z);
    let inter_max_z = max_z.min(other.max_z);

    if inter_min_x >= inter_max_x || inter_min_z >= inter_max_z {
        return 0.0;
    }

    let inter_area = (inter_max_x - inter_min_x) * (inter_max_z - inter_min_z);
    let area_a = (max_x - min_x) * (max_z - min_z);
    let area_b = (other.max_x - other.min_x) * (other.max_z - other.min_z);
    let union = area_a + area_b - inter_area;

    if union <= 0.0 {
        0.0
    } else {
        inter_area / union
    }
}
