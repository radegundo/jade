use bevy::prelude::*;

pub mod relative_map;

#[derive(Resource)]
pub struct Map {
    pub sectors: Vec<Sector>,
}

pub struct Sector {
    pub walls: Vec<LineDef>,
    pub floor_height: f32,
    pub ceiling_height: f32,
    pub floor_texture: Handle<Image>,
    pub ceiling_texture: Handle<Image>,
    pub id: usize,
}

pub struct LineDef {
    pub start: Vec2,
    pub end: Vec2,
    pub front_side_def: SideDef,
    pub back_side_def: Option<SideDef>,
    pub id: f32,
}

pub struct SideDef {
    pub textures: SideDefTextures,
    //Sector the side def is facing
    pub facing: usize,
}

pub struct SideDefTextures {
    pub upper: Option<Handle<Image>>,
    pub middle: Option<Handle<Image>>,
    pub lower: Option<Handle<Image>>,
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
pub fn wall(x0: f32, y0: f32, x1: f32, y1: f32, texture: Handle<Image>, id: f32) -> LineDef {
    LineDef {
        start: Vec2::new(x0, y0),
        end: Vec2::new(x1, y1),
        front_side_def: SideDef::new(
            SideDefTextures { upper: None, middle: Some(texture), lower: None },
            id.trunc() as usize
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
    id: f32,
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
        id: usize,
        texture: Handle<Image>
    ) -> Self {
        let wall = wall(x0, y0, x1, y1, texture, (self.id as f32) + (id as f32));
        self.walls.push(wall);
        self
    }
    pub fn portal(
        mut self,
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        id: usize,
        upper_texture: Handle<Image>,
        lower_texture: Handle<Image>,
        front_sector: usize,
        back_sector: usize
    ) -> Self {
        let portal = portal(
            x0,
            y0,
            x1,
            y1,
            upper_texture,
            lower_texture,
            (self.id as f32) + (id as f32),
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

//-------------- MAP FUNCTIONS

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
