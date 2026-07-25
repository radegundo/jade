use bevy::{ camera::visibility::RenderLayers, prelude::* };

use crate::map::relative_map::RelativeMapPlugin;

pub mod relative_map;

//------------------------------MAP PLUGIN-------------------------
pub struct MapPlugin;
impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_gizmo_layers)
            .add_systems(Startup, setup_map)
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
    pub floor_height: f32,
    pub ceiling_height: f32,
    pub floor_texture: Handle<Image>,
    pub ceiling_texture: Handle<Image>,
    pub id: usize,
}

/// Unique identifier for a wall: sector + index within that sector
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
    /// Sector the side def is facing
    pub facing: usize,
}

#[derive(Clone)]
pub struct SideDefTextures {
    pub upper: Option<Handle<Image>>,
    pub middle: Option<Handle<Image>>,
    pub lower: Option<Handle<Image>>,
}
//------------MINI MAP----------------------------

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
    let mut builder = SectorBuilder::new(
        id,
        floor_height,
        ceiling_height,
        floor_texture,
        ceiling_texture
    );
    builder
        .wall(x0, y0, x1, y0, 0, wall_texture.clone())
        .wall(x1, y0, x1, y1, 1, wall_texture.clone())
        .wall(x1, y1, x0, y1, 2, wall_texture.clone())
        .wall(x0, y1, x0, y0, 3, wall_texture.clone())
        .build()
}

//--------------LINE DEF BUILDING FUNCTIONS-------------
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

//-------------SIDE DEF BUILDING FUNCTIONS--------------
impl SideDef {
    pub fn new(textures: SideDefTextures, facing: usize) -> Self {
        Self { textures, facing }
    }
}

// ------------ API FOR BUILDING SECTORS ---------------
pub struct SectorBuilder {
    walls: Vec<LineDef>,
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
        let wall = wall(x0, y0, x1, y1, texture, wall_id);
        self.walls.push(wall);
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
        let portal = portal(
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
        self.walls.push(portal);
        self
    }

    pub fn build(self) -> Sector {
        Sector {
            walls: self.walls,
            floor_height: self.floor_height,
            ceiling_height: self.ceiling_height,
            floor_texture: self.floor_texture,
            ceiling_texture: self.ceiling_texture,
            id: self.id,
        }
    }
}
//---------------SYSTEMS----------------------

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
//-------------- MAP FUNCTIONS------------------------

pub fn test_map(asset_server: Res<AssetServer>) -> Map {
    let wall_tex: Handle<Image> = asset_server.load("texture.png");
    let floor_tex: Handle<Image> = asset_server.load("floor_texture.png");
    let ceil_tex: Handle<Image> = asset_server.load("floor_texture.png");

    Map {
        sectors: vec![
            // Sector 0: Main room, 100x100, floor at 0, ceiling at 20
            SectorBuilder::new(0, 0.0, 20.0, floor_tex.clone(), ceil_tex.clone())
                .wall(0.0, 0.0, 100.0, 0.0, 0, wall_tex.clone()) // Bottom: (0,0) → (100,0)
                .wall(100.0, 0.0, 100.0, 40.0, 1, wall_tex.clone()) // Right lower: (100,0) → (100,40)
                .portal(
                    100.0,
                    40.0, // Portal start
                    100.0,
                    60.0, // Portal end
                    2, // wall index
                    wall_tex.clone(), // upper texture
                    wall_tex.clone(), // lower texture
                    0, // front sector (this)
                    1 // back sector (sector 1)
                )
                .wall(100.0, 60.0, 100.0, 100.0, 3, wall_tex.clone()) // Right upper: (100,60) → (100,100)
                .wall(100.0, 100.0, 0.0, 100.0, 4, wall_tex.clone()) // Top: (100,100) → (0,100)
                .wall(0.0, 100.0, 0.0, 0.0, 5, wall_tex.clone()) // Left: (0,100) → (0,0)
                .build(),

            // Sector 1: Corridor/room to the right of the portal
            // Extends from x=100 to x=140, y=40 to y=60
            // Portal is the LEFT edge (x=100), shared with sector 0
            SectorBuilder::new(1, 0.0, 20.0, floor_tex.clone(), ceil_tex.clone())
                .wall(100.0, 40.0, 140.0, 40.0, 0, wall_tex.clone()) // Bottom: (100,40) → (140,40)
                .wall(140.0, 40.0, 140.0, 60.0, 1, wall_tex.clone()) // Right: (140,40) → (140,60)
                .wall(140.0, 60.0, 100.0, 60.0, 2, wall_tex.clone()) // Top: (140,60) → (100,60)
                .portal(
                    100.0,
                    60.0, // Portal start (top of shared edge)
                    100.0,
                    40.0, // Portal end (bottom of shared edge)
                    3, // wall index
                    wall_tex.clone(), // upper
                    wall_tex.clone(), // lower
                    1, // front sector (this)
                    0 // back sector (sector 0)
                )
                .build()
        ],
    }
}
// pub fn test_map(asset_server: Res<AssetServer>) -> Map {
//     let texture: Handle<Image> = asset_server.load("texture.png");
//     Map {
//         sectors: vec![
//             rect_sector(
//                 0,
//                 0.0,
//                 0.0,
//                 100.0,
//                 100.0,
//                 0.0,
//                 10.0,
//                 texture.clone(),
//                 texture.clone(),
//                 texture.clone()
//             )
//         ],
//     }
// }
