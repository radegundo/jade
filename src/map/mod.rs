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
        0,
        floor_height,
        ceiling_height,
        floor_texture,
        ceiling_texture
    );
    builder
        .wall(x0, y0, x1, y0, 0, wall_texture.clone())
        .wall(x1, y0, x1, y1, 1, wall_texture.clone())
        .wall(x1, y1, x0, y1, 2, wall_texture.clone())
        .wall(x0, y1, x0, y0, 3, wall_texture.clone());
    builder.build()
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
        &mut self,
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        id: usize,
        texture: Handle<Image>
    ) -> &mut Self {
        self.walls.push(wall(x0, y0, x1, y1, texture, (self.id as f32) + (id as f32)));
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
    let texture: Handle<Image> = asset_server.load("texture.png");
    Map {
        sectors: vec![
            rect_sector(
                0.0,
                0.0,
                100.0,
                100.0,
                0.0,
                10.0,
                texture.clone(),
                texture.clone(),
                texture.clone()
            )
        ],
    }
}
