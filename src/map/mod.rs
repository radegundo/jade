use bevy::{ camera::visibility::RenderLayers, prelude::* };
use crate::{ EYE_OFFSET, PlayerCameraCache, ViewInfo, map::relative_map::RelativeMapPlugin };

pub mod relative_map;

//------------------------------MAP PLUGIN-------------------------
pub struct MapPlugin;
impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_gizmo_layers)
            .add_systems(Startup, setup_map)
            .add_systems(Update, update_eye_height)
            .init_gizmo_group::<MapGizmos>()
            .add_plugins(RelativeMapPlugin);
    }
}

//------------------------------MAP DATA STRUCTURES-----------------

#[derive(Resource)]
pub struct Map {
    pub sectors: Vec<Sector>,
}

#[derive(Clone)]
pub struct Sector {
    pub walls: Vec<LineDef>,
    pub obstacles: Vec<Obstacle>,
    pub floor_height: f32,
    pub ceiling_height: f32,
    pub floor_texture: Handle<Image>,
    pub ceiling_texture: Handle<Image>,
    pub id: usize,
}

/// An obstacle is a 2D polygon with a vertical extent that lives inside
/// a sector. It does not form sector boundaries — it floats within the space.
#[derive(Clone)]
pub struct Obstacle {
    pub id: usize,
    pub edges: Vec<LineDef>,
    pub bottom: f32,
    pub top: f32,
    pub texture: Handle<Image>,
}

/// Uniquely identifies a wall within the map.
/// sector: which sector owns this wall
/// index: position in sector.walls (or obstacle.edges for obstacles)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WallId {
    pub sector: usize,
    pub index: usize,
}

impl WallId {
    pub fn new(sector: usize, index: usize) -> Self {
        Self { sector, index }
    }
}

#[derive(Clone)]
pub struct LineDef {
    pub start: Vec2,
    pub end: Vec2,
    pub front_side_def: SideDef,
    pub back_side_def: Option<SideDef>,
    pub id: WallId,
}

#[derive(Clone)]
pub struct SideDef {
    pub textures: SideDefTextures,
    /// Which sector this side faces toward
    pub facing: usize,
}

#[derive(Clone)]
pub struct SideDefTextures {
    /// Rendered above a portal opening (front ceiling higher than back ceiling)
    pub upper: Option<Handle<Image>>,
    /// Rendered on solid walls
    pub middle: Option<Handle<Image>>,
    /// Rendered below a portal opening (back floor higher than front floor)
    pub lower: Option<Handle<Image>>,
}

//-----------------------------GIZMO CONFIGS--------------------------------

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct MapGizmos;

fn setup_gizmo_layers(mut config_store: ResMut<GizmoConfigStore>) {
    let (config, _) = config_store.config_mut::<MapGizmos>();
    config.render_layers = RenderLayers::layer(1);
}

//-----------------------------MAP SETUP--------------------------------

fn setup_map(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(test_map(asset_server));
}

//------------HELPER FUNCTIONS FOR SECTOR BUILDING----------------

pub fn rect_sector(
    id: usize,
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    floor_height: f32,
    ceiling_height: f32,
    floor_texture: Handle<Image>,
    wall_texture: Handle<Image>,
    ceiling_texture: Handle<Image>
) -> Sector {
    SectorBuilder::new(id, floor_height, ceiling_height, floor_texture, ceiling_texture)
        .wall(x0, y0, x1, y0, 0, wall_texture.clone())
        .wall(x1, y0, x1, y1, 1, wall_texture.clone())
        .wall(x1, y1, x0, y1, 2, wall_texture.clone())
        .wall(x0, y1, x0, y0, 3, wall_texture.clone())
        .build()
}

pub fn wall(x0: f32, y0: f32, x1: f32, y1: f32, texture: Handle<Image>, id: WallId) -> LineDef {
    LineDef {
        start: Vec2::new(x0, y0),
        end: Vec2::new(x1, y1),
        front_side_def: SideDef::new(
            SideDefTextures { upper: None, middle: Some(texture), lower: None },
            id.sector
        ),
        back_side_def: None,
        id,
    }
}

pub fn portal(
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    upper_texture: Handle<Image>,
    lower_texture: Handle<Image>,
    id: WallId,
    front_sector: usize,
    back_sector: usize
) -> LineDef {
    LineDef {
        start: Vec2::new(x0, y0),
        end: Vec2::new(x1, y1),
        front_side_def: SideDef::new(
            SideDefTextures {
                upper: Some(upper_texture.clone()),
                middle: None,
                lower: Some(lower_texture.clone()),
            },
            front_sector
        ),
        back_side_def: Some(
            SideDef::new(
                SideDefTextures {
                    upper: Some(upper_texture.clone()),
                    middle: None,
                    lower: Some(lower_texture.clone()),
                },
                back_sector
            )
        ),
        id,
    }
}

impl SideDef {
    pub fn new(textures: SideDefTextures, facing: usize) -> Self {
        Self { textures, facing }
    }
}

//------------- OBSTACLE BUILDER ---------------

pub struct ObstacleBuilder {
    edges: Vec<LineDef>,
    bottom: f32,
    top: f32,
    id: usize,
    sector_id: usize,
    wall_counter: usize,
    texture: Handle<Image>,
}

impl ObstacleBuilder {
    pub fn new(id: usize, sector_id: usize, bottom: f32, top: f32, texture: Handle<Image>) -> Self {
        Self {
            edges: Vec::new(),
            bottom,
            top,
            id,
            sector_id,
            wall_counter: 0,
            texture,
        }
    }

    pub fn edge(mut self, x0: f32, y0: f32, x1: f32, y1: f32) -> Self {
        // Obstacle edges use a WallId index offset of 1000+
        // to avoid colliding with sector wall indices (0, 1, 2...)
        let wall_id = WallId::new(self.sector_id, 1000 + self.id * 100 + self.wall_counter);
        let edge = wall(x0, y0, x1, y1, self.texture.clone(), wall_id);
        self.edges.push(edge);
        self.wall_counter += 1;
        self
    }

    pub fn build(self) -> Obstacle {
        Obstacle {
            id: self.id,
            edges: self.edges,
            bottom: self.bottom,
            top: self.top,
            texture: self.texture,
        }
    }
}

/// Builds a rectangular box obstacle.
/// Edges are wound CLOCKWISE so normals face OUTWARD.
/// Sector walls are wound CCW (inward normals) because the player
/// is inside them. Obstacle edges are wound CW (outward normals)
/// because the player is outside them.
pub fn rect_obstacle(
    id: usize,
    sector_id: usize,
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    bottom: f32,
    top: f32,
    texture: Handle<Image>
) -> Obstacle {
    ObstacleBuilder::new(id, sector_id, bottom, top, texture)
        .edge(x1, y0, x0, y0) // bottom (reversed)
        .edge(x0, y0, x0, y1) // left   (reversed)
        .edge(x0, y1, x1, y1) // top    (reversed)
        .edge(x1, y1, x1, y0) // right  (reversed)
        .build()
}

//------------- SECTOR BUILDER ---------------

pub struct SectorBuilder {
    walls: Vec<LineDef>,
    obstacles: Vec<Obstacle>,
    floor_height: f32,
    ceiling_height: f32,
    id: usize,
    floor_texture: Handle<Image>,
    ceiling_texture: Handle<Image>,
}

impl SectorBuilder {
    pub fn new(
        id: usize,
        floor_height: f32,
        ceiling_height: f32,
        floor_texture: Handle<Image>,
        ceiling_texture: Handle<Image>
    ) -> Self {
        SectorBuilder {
            walls: Vec::new(),
            obstacles: Vec::new(),
            floor_height,
            ceiling_height,
            id,
            floor_texture,
            ceiling_texture,
        }
    }

    pub fn wall(
        mut self,
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        wall_index: usize,
        texture: Handle<Image>
    ) -> Self {
        let wall_id = WallId::new(self.id, wall_index);
        let w = wall(x0, y0, x1, y1, texture, wall_id);
        self.walls.push(w);
        self
    }

    pub fn portal(
        mut self,
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        wall_index: usize,
        upper_texture: Handle<Image>,
        lower_texture: Handle<Image>,
        front_sector: usize,
        back_sector: usize
    ) -> Self {
        let wall_id = WallId::new(self.id, wall_index);
        let p = portal(
            x0,
            y0,
            x1,
            y1,
            upper_texture,
            lower_texture,
            wall_id,
            front_sector,
            back_sector
        );
        self.walls.push(p);
        self
    }

    pub fn obstacle(mut self, obstacle: Obstacle) -> Self {
        self.obstacles.push(obstacle);
        self
    }

    pub fn rect_obstacle(
        self,
        id: usize,
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        bottom: f32,
        top: f32,
        texture: Handle<Image>
    ) -> Self {
        let obs = rect_obstacle(id, self.id, x0, y0, x1, y1, bottom, top, texture);
        self.obstacle(obs)
    }

    pub fn build(self) -> Sector {
        Sector {
            walls: self.walls,
            obstacles: self.obstacles,
            floor_height: self.floor_height,
            ceiling_height: self.ceiling_height,
            floor_texture: self.floor_texture,
            ceiling_texture: self.ceiling_texture,
            id: self.id,
        }
    }
}

//---------------SYSTEMS----------------------

/// Point-in-polygon test using ray casting.
/// Counts how many sector walls a horizontal ray from point crosses.
/// Odd count = inside, even = outside.
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

/// Smoothly moves view_info.eye_height toward the current sector's floor
/// plus EYE_OFFSET. This makes stepping between different-height sectors
/// feel smooth instead of snapping.
pub fn update_eye_height(
    player_cache: Res<PlayerCameraCache>,
    map: Res<Map>,
    mut view_info: ResMut<ViewInfo>,
    time: Res<Time>
) {
    let pos = player_cache.transform.translation.truncate();
    if let Some(sector_idx) = find_player_sector(pos, &map) {
        let sector = &map.sectors[sector_idx];
        let target_eye_height = sector.floor_height + EYE_OFFSET;
        let speed = 8.0;
        view_info.eye_height =
            view_info.eye_height +
            (target_eye_height - view_info.eye_height) * (speed * time.delta_secs()).min(1.0);
    }
}

//-------------- MAP DATA ------------------------

pub fn test_map(asset_server: Res<AssetServer>) -> Map {
    let wall_tex: Handle<Image> = asset_server.load("texture.png");
    let floor_tex: Handle<Image> = asset_server.load("floor_texture.png");
    let ceil_tex: Handle<Image> = asset_server.load("floor_texture.png");

    Map {
        sectors: vec![
            // Sector 0: Main room — 100x100, floor at 0, ceiling at 20
            SectorBuilder::new(0, 0.0, 20.0, floor_tex.clone(), ceil_tex.clone())
                .wall(0.0, 0.0, 100.0, 0.0, 0, wall_tex.clone())
                .wall(100.0, 0.0, 100.0, 40.0, 1, wall_tex.clone())
                .portal(100.0, 40.0, 100.0, 60.0, 2, wall_tex.clone(), wall_tex.clone(), 0, 1)
                .wall(100.0, 60.0, 100.0, 100.0, 3, wall_tex.clone())
                .wall(100.0, 100.0, 0.0, 100.0, 4, wall_tex.clone())
                .wall(0.0, 100.0, 0.0, 0.0, 5, wall_tex.clone())
                // Box sitting on the floor (0 → 8)
                .rect_obstacle(0, 40.0, 40.0, 50.0, 50.0, 0.0, 8.0, wall_tex.clone())
                // Floating platform (5 → 7)
                .rect_obstacle(1, 60.0, 70.0, 80.0, 90.0, 5.0, 7.0, wall_tex.clone())
                .build(),

            // Sector 1: Corridor — raised floor (10), portal back to sector 0
            SectorBuilder::new(1, 10.0, 20.0, floor_tex.clone(), ceil_tex.clone())
                .wall(100.0, 40.0, 140.0, 40.0, 0, wall_tex.clone())
                .wall(140.0, 40.0, 140.0, 60.0, 1, wall_tex.clone())
                .wall(140.0, 60.0, 100.0, 60.0, 2, wall_tex.clone())
                .portal(100.0, 60.0, 100.0, 40.0, 3, wall_tex.clone(), wall_tex.clone(), 1, 0)
                .build()
        ],
    }
}
