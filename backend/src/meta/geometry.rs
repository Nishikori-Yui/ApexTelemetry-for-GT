// Track geometry lookup and SVG extraction.

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use tracing::warn;

const VENDOR_DIR: &str = "vendor";
const GT7TRACKS_DIR: &str = "GT7Tracks";
const GT7TRACKS_DUMPS_DIR: &str = "dumps";

#[derive(Clone, Debug)]
pub struct TrackSvg {
    pub view_box: String,
    pub path_d: String,
    pub points_count: usize,
    pub simplified: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct TrackBounds {
    pub min_x: f32,
    pub max_x: f32,
    pub min_z: f32,
    pub max_z: f32,
}

pub struct TrackGeometryIndex {
    pub(crate) dumps_dir: PathBuf,
    pub(crate) bounds: HashMap<i32, TrackBounds>,
    pub(crate) svg_cache: Mutex<HashMap<i32, TrackSvg>>,
}

impl TrackGeometryIndex {
    pub fn load(data_dir: &Path) -> Self {
        let dumps_dir = data_dir.join(VENDOR_DIR).join(GT7TRACKS_DIR).join(GT7TRACKS_DUMPS_DIR);
        let bounds = load_track_bounds(&dumps_dir);
        Self {
            dumps_dir,
            bounds,
            svg_cache: Mutex::new(HashMap::new()),
        }
    }

    pub fn has_geometry(&self, track_id: i32) -> bool {
        let path = self.dumps_dir.join(format!("{track_id}.csv"));
        path.is_file()
    }

    pub fn get_geometry_path(&self, track_id: i32) -> Option<PathBuf> {
        let path = self.dumps_dir.join(format!("{track_id}.csv"));
        if path.is_file() {
            Some(path)
        } else {
            None
        }
    }

    pub fn get_geometry_svg(&self, track_id: i32) -> Option<TrackSvg> {
        if let Ok(cache) = self.svg_cache.lock() {
            if let Some(svg) = cache.get(&track_id) {
                return Some(svg.clone());
            }
        }

        let path = self.get_geometry_path(track_id)?;
        let svg = build_track_svg(&path)?;

        if let Ok(mut cache) = self.svg_cache.lock() {
            cache.insert(track_id, svg.clone());
        }

        Some(svg)
    }
}

fn load_track_bounds(dumps_dir: &Path) -> HashMap<i32, TrackBounds> {
    let mut bounds = HashMap::new();
    if !dumps_dir.is_dir() {
        warn!(dumps_dir = %dumps_dir.display(), "GT7Tracks dumps directory not found");
        return bounds;
    }

    let entries = match fs::read_dir(dumps_dir) {
        Ok(entries) => entries,
        Err(err) => {
            warn!(?err, dumps_dir = %dumps_dir.display(), "failed to read GT7Tracks dumps directory");
            return bounds;
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(err) => {
                warn!(?err, "failed to read GT7Tracks entry");
                continue;
            }
        };
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("csv") {
            continue;
        }
        let track_id = match path.file_stem().and_then(|stem| stem.to_str()) {
            Some(stem) => match stem.parse::<i32>() {
                Ok(value) => value,
                Err(_) => continue,
            },
            None => continue,
        };
        let file = match File::open(&path) {
            Ok(file) => file,
            Err(err) => {
                warn!(?err, path = %path.display(), "failed to open GT7Tracks dump");
                continue;
            }
        };
        let mut reader = BufReader::new(file);
        let mut line = String::new();
        let mut min_x = f32::MAX;
        let mut max_x = f32::MIN;
        let mut min_z = f32::MAX;
        let mut max_z = f32::MIN;
        let mut line_index = 0usize;

        loop {
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {}
                Err(err) => {
                    warn!(?err, path = %path.display(), "failed reading GT7Tracks dump");
                    break;
                }
            }

            line_index += 1;
            if line_index == 1 {
                continue;
            }
            let mut parts = line.trim_end().split(',');
            let _track_col = parts.next();
            let x = parts.next().and_then(|value| value.parse::<f32>().ok());
            let z = parts.next().and_then(|value| value.parse::<f32>().ok());
            if let (Some(x), Some(z)) = (x, z) {
                min_x = min_x.min(x);
                max_x = max_x.max(x);
                min_z = min_z.min(z);
                max_z = max_z.max(z);
            }
        }

        if min_x < max_x && min_z < max_z {
            bounds.insert(
                track_id,
                TrackBounds {
                    min_x,
                    max_x,
                    min_z,
                    max_z,
                },
            );
        }
    }

    bounds
}

pub fn build_track_svg(path: &Path) -> Option<TrackSvg> {
    let points = read_track_points(path)?;
    if points.len() < 2 {
        return None;
    }

    let simplify_step = 5usize;
    let mut sampled: Vec<(f32, f32)> = Vec::new();
    for (idx, point) in points.iter().enumerate() {
        if idx % simplify_step == 0 {
            sampled.push(*point);
        }
    }
    if let Some(last) = points.last() {
        if sampled.last() != Some(last) {
            sampled.push(*last);
        }
    }

    let (min_x, max_x, min_z, max_z) = sampled.iter().fold(
        (f32::MAX, f32::MIN, f32::MAX, f32::MIN),
        |acc, (x, z)| (acc.0.min(*x), acc.1.max(*x), acc.2.min(*z), acc.3.max(*z)),
    );
    if min_x >= max_x || min_z >= max_z {
        return None;
    }

    let width = max_x - min_x;
    let height = max_z - min_z;
    let canvas = 1000.0;
    let pad = 40.0;
    let scale = ((canvas - pad * 2.0) / width).min((canvas - pad * 2.0) / height);

    let mut path_d = String::new();
    for (index, (x, z)) in sampled.iter().enumerate() {
        let sx = (x - min_x) * scale + pad;
        let sy = (max_z - z) * scale + pad;
        if index == 0 {
            path_d.push_str(&format!("M {:.2} {:.2}", sx, sy));
        } else {
            path_d.push_str(&format!(" L {:.2} {:.2}", sx, sy));
        }
    }

    Some(TrackSvg {
        view_box: format!("0 0 {:.0} {:.0}", canvas, canvas),
        path_d,
        points_count: sampled.len(),
        simplified: simplify_step > 1,
    })
}

fn read_track_points(path: &Path) -> Option<Vec<(f32, f32)>> {
    let file = File::open(path).ok()?;
    let mut reader = BufReader::new(file);
    let mut line = String::new();
    let mut points: Vec<(f32, f32)> = Vec::new();
    let mut line_index = 0usize;

    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {}
            Err(_) => break,
        }
        line_index += 1;
        if line_index == 1 {
            continue;
        }
        let mut parts = line.trim_end().split(',');
        let _track_col = parts.next();
        let x = parts.next().and_then(|value| value.parse::<f32>().ok());
        let z = parts.next().and_then(|value| value.parse::<f32>().ok());
        if let (Some(x), Some(z)) = (x, z) {
            points.push((x, z));
        }
    }

    if points.is_empty() {
        None
    } else {
        Some(points)
    }
}
