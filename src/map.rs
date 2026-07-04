use bevy::prelude::*;


#[derive(Resource)]
pub struct Map{
   pub walls: Vec<Wall>,
}

pub struct Wall{
    pub start: Vec2,
    pub end: Vec2
}

impl Wall{
    pub fn new(x0: f32, y0: f32, x1:f32, y1:f32) -> Self{
        Wall { start: Vec2::new(x0, y0), end: Vec2::new(x1, y1) }
    }
}

pub fn draw_walls(map: Res<Map>, mut gizmos: Gizmos){
    for wall in &map.walls{
        gizmos.line(wall.start.extend(0.0), wall.end.extend(0.0), Color::WHITE);
    }
}