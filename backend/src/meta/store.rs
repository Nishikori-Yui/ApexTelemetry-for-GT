// Metadata store for cars/tracks and geometry lookups.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use tracing::{info, warn};

use super::geometry::{TrackBounds, TrackGeometryIndex, TrackSvg};

const CARS_CSV: &str = include_str!("data/cars.csv");
const MAKERS_CSV: &str = include_str!("data/maker.csv");
const COURSES_CSV: &str = include_str!("data/course.csv");

#[derive(Clone, Debug)]
pub struct CarMeta {
    pub id: i32,
    pub name: String,
    pub manufacturer: Option<String>,
}

#[derive(Clone, Debug)]
pub struct TrackMeta {
    pub id: i32,
    pub base_id: Option<i32>,
    pub name: String,
    pub layout_number: Option<i32>,
    pub is_reverse: Option<bool>,
}

pub struct MetadataStore {
    cars: HashMap<i32, CarMeta>,
    tracks: HashMap<i32, TrackMeta>,
    tracks_by_base: HashMap<i32, i32>,
    geometry: TrackGeometryIndex,
}

impl MetadataStore {
    pub fn load(data_dir: &Path) -> Self {
        let makers = load_makers();
        let cars = load_cars(&makers);
        let tracks = load_tracks();
        let tracks_by_base = build_track_base_index(&tracks);
        let geometry = TrackGeometryIndex::load(data_dir);

        info!(
            car_count = cars.len(),
            track_count = tracks.len(),
            geometry_tracks = geometry.bounds.len(),
            dumps_dir = %geometry.dumps_dir.display(),
            "metadata loaded"
        );

        Self {
            cars,
            tracks,
            tracks_by_base,
            geometry,
        }
    }

    pub fn get_car_info(&self, id: i32) -> Option<&CarMeta> {
        self.cars.get(&id)
    }

    pub fn get_car_name(&self, id: i32) -> Option<&str> {
        self.cars.get(&id).map(|car| car.name.as_str())
    }

    pub fn get_track_info(&self, id_or_base: i32) -> Option<&TrackMeta> {
        if let Some(track) = self.tracks.get(&id_or_base) {
            return Some(track);
        }
        self.tracks_by_base
            .get(&id_or_base)
            .and_then(|track_id| self.tracks.get(track_id))
    }

    pub fn get_track_name(&self, id_or_base: i32) -> Option<&str> {
        self.get_track_info(id_or_base)
            .map(|track| track.name.as_str())
    }

    pub fn has_track_geometry(&self, track_id: i32) -> bool {
        self.geometry.has_geometry(track_id)
    }

    pub fn get_track_geometry_path(&self, track_id: i32) -> Option<PathBuf> {
        self.geometry.get_geometry_path(track_id)
    }

    pub fn track_bounds(&self) -> &HashMap<i32, TrackBounds> {
        &self.geometry.bounds
    }

    pub fn get_track_geometry_svg(&self, track_id: i32) -> Option<TrackSvg> {
        self.geometry.get_geometry_svg(track_id)
    }
}

fn load_makers() -> HashMap<i32, String> {
    let mut makers = HashMap::new();
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(MAKERS_CSV.as_bytes());
    for record in reader.records() {
        let record = match record {
            Ok(record) => record,
            Err(err) => {
                warn!(?err, "maker csv parse failed");
                continue;
            }
        };
        let id = parse_i32(record.get(0));
        let name = record.get(1).map(str::trim).filter(|v| !v.is_empty());
        if let (Some(id), Some(name)) = (id, name) {
            makers.insert(id, name.to_string());
        }
    }
    makers
}

fn load_cars(makers: &HashMap<i32, String>) -> HashMap<i32, CarMeta> {
    let mut cars = HashMap::new();
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(CARS_CSV.as_bytes());
    for record in reader.records() {
        let record = match record {
            Ok(record) => record,
            Err(err) => {
                warn!(?err, "cars csv parse failed");
                continue;
            }
        };
        let id = parse_i32(record.get(0));
        let name = record.get(1).map(str::trim).filter(|v| !v.is_empty());
        let maker_id = parse_i32(record.get(2));
        if let (Some(id), Some(name)) = (id, name) {
            let manufacturer = maker_id.and_then(|maker_id| makers.get(&maker_id).cloned());
            cars.insert(
                id,
                CarMeta {
                    id,
                    name: name.to_string(),
                    manufacturer,
                },
            );
        }
    }
    cars
}

fn load_tracks() -> HashMap<i32, TrackMeta> {
    let mut tracks = HashMap::new();
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(COURSES_CSV.as_bytes());
    for record in reader.records() {
        let record = match record {
            Ok(record) => record,
            Err(err) => {
                warn!(?err, "course csv parse failed");
                continue;
            }
        };
        let id = parse_i32(record.get(0));
        let name = record.get(1).map(str::trim).filter(|v| !v.is_empty());
        let base_id = parse_i32(record.get(2));
        let layout_number = parse_i32(record.get(14));
        let is_reverse = parse_bool(record.get(15));
        if let (Some(id), Some(name)) = (id, name) {
            tracks.insert(
                id,
                TrackMeta {
                    id,
                    base_id,
                    name: name.to_string(),
                    layout_number,
                    is_reverse,
                },
            );
        }
    }
    tracks
}

fn build_track_base_index(tracks: &HashMap<i32, TrackMeta>) -> HashMap<i32, i32> {
    let mut index: HashMap<i32, (i32, bool, i32)> = HashMap::new();
    for track in tracks.values() {
        let base_id = match track.base_id {
            Some(id) => id,
            None => continue,
        };
        let layout = track.layout_number.unwrap_or(i32::MAX);
        let reverse = track.is_reverse.unwrap_or(false);
        let candidate = (layout, reverse, track.id);
        match index.get(&base_id) {
            Some(existing) => {
                let replace = candidate.0 < existing.0
                    || (candidate.0 == existing.0 && candidate.1 == false && existing.1 == true);
                if replace {
                    index.insert(base_id, candidate);
                }
            }
            None => {
                index.insert(base_id, candidate);
            }
        }
    }
    index.into_iter().map(|(base_id, (_, _, id))| (base_id, id)).collect()
}

fn parse_i32(value: Option<&str>) -> Option<i32> {
    value.and_then(|value| value.trim().parse::<i32>().ok())
}

fn parse_bool(value: Option<&str>) -> Option<bool> {
    match value.map(str::trim) {
        Some("1") => Some(true),
        Some("0") => Some(false),
        _ => None,
    }
}
