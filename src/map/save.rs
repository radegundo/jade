//! Load maps saved by the **jade-ed** editor.
//!
//! Maps live in `assets/maps/<name>.json`, produced by `jade-ed`'s `save.rs`.
//! The `Save*` structs below are a byte-for-byte copy of that project's disk
//! model — the JSON is the shared contract, so keep the two files in sync.
//!
//! Which map is loaded is controlled by [`MAP_NAME`] (change it to switch
//! levels) and `setup_map` falls back to `test_map` when the file is missing.

use bevy::math::Vec2;
use bevy::prelude::*;
use serde::{ Deserialize, Serialize };

use super::{ LineDef, Map, Obstacle, Sector, SideDef, SideDefTextures, WallId };

//------------------------------CONFIG--------------------------------

/// Folder (relative to the crate root) maps are read from.
pub const MAPS_DIR: &str = "assets/maps";

/// Name of the map to load at startup, e.g. `"sample"` → `assets/maps/sample.json`.
/// Set to `""` to always fall back to the built-in `test_map`.
pub const MAP_NAME: &str = "test";

//------------------------------DISK MODEL---------------------------

/// Current save-format version. Must match `jade-ed`'s `SAVE_VERSION`.
pub const SAVE_VERSION: u32 = 1;

#[derive(Serialize, Deserialize, Clone)]
pub struct SaveMap {
    /// `#[serde(default)]` lets files saved before versioning load as v0.
    #[serde(default)]
    pub version: u32,
    pub vertices: Vec<Vec2>,
    pub sectors: Vec<SaveSector>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SaveSector {
    pub walls: Vec<SaveLine>,
    pub obstacles: Vec<SaveObstacle>,
    pub floor_height: f32,
    pub ceiling_height: f32,
    pub floor_texture: String,
    pub ceiling_texture: String,
    pub id: usize,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SaveLine {
    pub start_idx: usize,
    pub end_idx: usize,
    pub front: SaveSide,
    pub back: Option<SaveSide>, // Some => this line is a portal
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SaveSide {
    pub upper: Option<String>,
    pub middle: Option<String>,
    pub lower: Option<String>,
    pub facing: usize, // the sector id this side belongs to
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SaveObstacle {
    pub id: usize,
    pub edges: Vec<SaveLine>,
    pub bottom: f32,
    pub top: f32,
    pub side_texture: String,
    pub top_texture: String,
    pub bottom_texture: String,
}

//------------------------------LOADING------------------------------

/// Sanitize a map name and build its `.json` path (mirrors `jade-ed`).
pub fn map_path(name: &str) -> String {
    let clean: String = name
        .trim()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' }
        })
        .collect();
    let file = if clean.is_empty() { "unnamed" } else { &clean };
    format!("{MAPS_DIR}/{file}.json")
}

/// Disk model → renderer `Map` (path strings become handles). `server.load`
/// deduplicates by path, so shared textures stay shared, matching `test_map`.
fn from_save(save: SaveMap, server: &AssetServer) -> Map {
    let vertices = save.vertices;
    let sectors = save.sectors
        .into_iter()
        .map(|s| Sector {
            walls: s.walls
                .into_iter()
                .enumerate()
                .map(|(i, w)| line_from_save(w, WallId::new(s.id, i), server))
                .collect(),
            obstacles: s.obstacles
                .into_iter()
                .map(|o| obstacle_from_save(o, s.id, server))
                .collect(),
            floor_height: s.floor_height,
            ceiling_height: s.ceiling_height,
            floor_texture: server.load(&s.floor_texture),
            ceiling_texture: server.load(&s.ceiling_texture),
            id: s.id,
        })
        .collect();

    Map { vertices, sectors }
}

fn line_from_save(w: SaveLine, id: WallId, server: &AssetServer) -> LineDef {
    let side = |s: SaveSide|
        SideDef::new(
            SideDefTextures {
                upper: s.upper.map(|t| server.load(&t)),
                middle: s.middle.map(|t| server.load(&t)),
                lower: s.lower.map(|t| server.load(&t)),
            },
            s.facing
        );

    LineDef {
        start_idx: w.start_idx,
        end_idx: w.end_idx,
        front_side_def: side(w.front),
        back_side_def: w.back.map(side),
        id,
    }
}

fn obstacle_from_save(o: SaveObstacle, sector_id: usize, server: &AssetServer) -> Obstacle {
    let edges = o.edges
        .into_iter()
        .enumerate()
        .map(|(i, w)| line_from_save(w, WallId::new(sector_id, i), server))
        .collect();

    Obstacle {
        id: o.id,
        edges,
        bottom: o.bottom,
        top: o.top,
        side_texture: server.load(&o.side_texture),
        top_texture: server.load(&o.top_texture),
        bottom_texture: server.load(&o.bottom_texture),
    }
}

/// Load and build the named map. `None` if the file is missing, unparsable,
/// or written by a newer format than this binary understands.
pub fn load_map(name: &str, server: &AssetServer) -> Option<Map> {
    let json = std::fs::read_to_string(map_path(name)).ok()?;
    let save: SaveMap = serde_json::from_str(&json).ok()?;
    if save.version > SAVE_VERSION {
        return None;
    }
    Some(from_save(save, server))
}
