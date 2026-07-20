pub mod absolute_map;
pub mod relative_map;
use bevy::prelude::*;

// ============================================================
// Core data structures
// ============================================================

#[derive(Resource)]
pub struct Sector {
    pub walls: Vec<LineDef>,
    pub ceiling_height: f32,
    pub floor_height: f32,
    pub id: usize,
    pub floor_texture: Handle<Image>,
    pub ceiling_texture: Handle<Image>,
    pub sector_type: SectorType,
    pub obstacle_ids: Option<Vec<usize>>,
}

pub enum SectorType {
    Sector,
    ObstacleSector,
}

#[derive(Resource)]
pub struct Map {
    pub sectors: Vec<Sector>,
    pub obstacle_sectors: Vec<Sector>,
}

#[derive(Default, Clone)]
pub struct LineDef {
    pub start: Vec2,
    pub end: Vec2,
    pub front_side_def: SideDef,
    pub back_side_def: Option<SideDef>,
    pub wall_id: WallId,
}

#[derive(Default, Clone)]
pub struct WallId(pub f32);

#[derive(Default, Clone)]
pub struct SideDef {
    pub upper_texture: Option<Handle<Image>>,
    pub middle_texture: Option<Handle<Image>>,
    pub lower_texture: Option<Handle<Image>>,
    pub sector: usize,
}

#[derive(Resource)]
pub struct MapWindow {
    pub id: Entity,
}

#[derive(Component)]
pub struct MapWindowMarker;

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum MapViewMode {
    #[default]
    Relative,
    Absolute,
}

// ============================================================
// LineDef constructors
// ============================================================

impl LineDef {
    /// A solid, one-sided wall (no back sector).
    pub fn new(x0: f32, y0: f32, x1: f32, y1: f32, texture: Handle<Image>, id: f32) -> Self {
        let front_side_def = SideDef {
            middle_texture: Some(texture),
            ..default()
        };
        LineDef {
            start: Vec2::new(x0, y0),
            end: Vec2::new(x1, y1),
            front_side_def,
            back_side_def: None,
            wall_id: WallId(id),
        }
    }
}

/// A solid, one-sided wall (no back sector). Free-function equivalent of `LineDef::new`.
pub fn solid_wall(x0: f32, y0: f32, x1: f32, y1: f32, texture: Handle<Image>) -> LineDef {
    LineDef::new(x0, y0, x1, y1, texture)
}

/// A two-sided portal wall connecting `front_sector` to `back_sector`.
/// Both sides default to see-through (no middle texture).
pub fn portal_wall(
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    front_sector: usize,
    back_sector: usize
) -> LineDef {
    LineDef {
        start: Vec2::new(x0, y0),
        end: Vec2::new(x1, y1),
        front_side_def: SideDef {
            middle_texture: None,
            sector: front_sector,
            ..default()
        },
        back_side_def: Some(SideDef {
            middle_texture: None,
            sector: back_sector,
            ..default()
        }),
    }
}

/// A two-sided portal wall that also carries upper/lower "step" textures,
/// for when the two connected sectors have different floor/ceiling heights.
pub fn portal_wall_with_steps(
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    front_sector: usize,
    back_sector: usize,
    upper_texture: Option<Handle<Image>>,
    lower_texture: Option<Handle<Image>>
) -> LineDef {
    LineDef {
        start: Vec2::new(x0, y0),
        end: Vec2::new(x1, y1),
        front_side_def: SideDef {
            middle_texture: None,
            upper_texture,
            lower_texture,
            sector: front_sector,
        },
        back_side_def: Some(SideDef {
            middle_texture: None,
            sector: back_sector,
            ..default()
        }),
    }
}

// ============================================================
// Sector constructors
// ============================================================

/// Build a closed rectangular sector from its bottom-left and top-right corners.
/// All four walls are solid and single-textured. Useful for quick test rooms.
pub fn rect_sector(
    self_index: usize,
    min: Vec2,
    max: Vec2,
    floor_height: f32,
    ceiling_height: f32,
    wall_texture: Handle<Image>,
    floor_texture: Handle<Image>,
    ceiling_texture: Handle<Image>
) -> Sector {
    SectorBuilder::new(
        self_index,
        floor_height,
        ceiling_height,
        floor_texture,
        ceiling_texture,
        None
    )
        .wall(min.x, min.y, max.x, min.y, wall_texture.clone())
        .wall(max.x, min.y, max.x, max.y, wall_texture.clone())
        .wall(max.x, max.y, min.x, max.y, wall_texture.clone())
        .wall(min.x, max.y, min.x, min.y, wall_texture)
        .build()
}

/// Builder-style struct for constructing a sector wall-by-wall.
pub struct SectorBuilder {
    walls: Vec<LineDef>,
    floor_height: f32,
    ceiling_height: f32,
    self_index: usize,
    floor_texture: Handle<Image>,
    ceiling_texture: Handle<Image>,
    obstacle_ids: Option<Vec<usize>>,
}

impl SectorBuilder {
    pub fn new(
        self_index: usize,
        floor_height: f32,
        ceiling_height: f32,
        floor_texture: Handle<Image>,
        ceiling_texture: Handle<Image>,
        obstacle_ids: Option<Vec<usize>>
    ) -> Self {
        SectorBuilder {
            walls: Vec::new(),
            floor_height,
            ceiling_height,
            self_index,
            floor_texture,
            ceiling_texture,
            obstacle_ids,
        }
    }

    /// Add a solid wall.
    pub fn wall(mut self, x0: f32, y0: f32, x1: f32, y1: f32, texture: Handle<Image>) -> Self {
        let mut wall = solid_wall(x0, y0, x1, y1, texture);
        wall.front_side_def.sector = self.self_index;
        self.walls.push(wall);
        self
    }

    /// Add a portal to `back_sector`.
    pub fn portal(mut self, x0: f32, y0: f32, x1: f32, y1: f32, back_sector: usize) -> Self {
        self.walls.push(portal_wall(x0, y0, x1, y1, self.self_index, back_sector));
        self
    }

    /// Add a portal to `back_sector` with upper/lower step textures.
    pub fn portal_with_steps(
        mut self,
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        back_sector: usize,
        upper_texture: Option<Handle<Image>>,
        lower_texture: Option<Handle<Image>>
    ) -> Self {
        self.walls.push(
            portal_wall_with_steps(
                x0,
                y0,
                x1,
                y1,
                self.self_index,
                back_sector,
                upper_texture,
                lower_texture
            )
        );
        self
    }

    pub fn build(self) -> Sector {
        Sector {
            walls: self.walls,
            floor_height: self.floor_height,
            ceiling_height: self.ceiling_height,
            id: self.self_index,
            floor_texture: self.floor_texture,
            ceiling_texture: self.ceiling_texture,
            sector_type: SectorType::Sector,
            obstacle_ids: self.obstacle_ids,
        }
    }
}

pub struct ObstacleSectorBuilder {
    walls: Vec<LineDef>,
    floor_height: f32,
    ceiling_height: f32,
    self_index: usize,
    floor_texture: Handle<Image>,
    ceiling_texture: Handle<Image>,
}

impl ObstacleSectorBuilder {
    pub fn new(
        self_index: usize,
        floor_height: f32,
        ceiling_height: f32,
        floor_texture: Handle<Image>,
        ceiling_texture: Handle<Image>
    ) -> Self {
        ObstacleSectorBuilder {
            walls: Vec::new(),
            floor_height,
            ceiling_height,
            self_index,
            floor_texture,
            ceiling_texture,
        }
    }

    /// Add a solid wall.
    pub fn wall(mut self, x0: f32, y0: f32, x1: f32, y1: f32, texture: Handle<Image>) -> Self {
        let mut wall = solid_wall(x0, y0, x1, y1, texture);
        wall.front_side_def.sector = self.self_index;
        self.walls.push(wall);
        self
    }

    pub fn build(self) -> Sector {
        Sector {
            walls: self.walls,
            floor_height: self.floor_height,
            ceiling_height: self.ceiling_height,
            id: self.self_index,
            floor_texture: self.floor_texture,
            ceiling_texture: self.ceiling_texture,
            sector_type: SectorType::ObstacleSector,
            obstacle_ids: None,
        }
    }
}

// ============================================================
// Point-in-sector / sector lookup
// ============================================================

pub fn point_in_sector(point: Vec2, sector: &Sector) -> bool {
    let mut inside = false;
    for wall in &sector.walls {
        let (x1, y1) = (wall.start.x, wall.start.y);
        let (x2, y2) = (wall.end.x, wall.end.y);
        let crosses = (y1 > point.y) != (y2 > point.y);
        if crosses {
            let x_intersect = x1 + ((point.y - y1) / (y2 - y1)) * (x2 - x1);
            if point.x < x_intersect {
                inside = !inside;
            }
        }
    }
    inside
}

pub fn find_player_sector(player_pos: Vec2, map: &Map) -> Option<usize> {
    for (i, sector) in map.sectors.iter().enumerate() {
        if point_in_sector(player_pos, sector) {
            return Some(i);
        }
    }
    None
}
