use bevy::prelude::*;
use crate::map::{ Map, SectorBuilder, rect_sector };

/// A single rectangular room, 200x200 units.
pub fn simple_room(asset_server: &AssetServer) -> Map {
    let wall = asset_server.load("texture.png");
    let floor = asset_server.load("floor_texture.png");
    let ceil = asset_server.load("floor_texture.png");

    Map {
        sectors: vec![rect_sector(0, 0.0, 0.0, 200.0, 200.0, 0.0, 20.0, floor, wall, ceil)],
    }
}

/// Two rooms separated by a portal (doorway).
/// Room A is 100x100, Room B is 80x80, offset, connected by a portal.
pub fn two_rooms(asset_server: &AssetServer) -> Map {
    let wall = asset_server.load("texture.png");
    let floor = asset_server.load("floor_texture.png");
    let ceil = asset_server.load("floor_texture.png");

    Map {
        sectors: vec![
            SectorBuilder::new(0, 0.0, 16.0, floor.clone(), ceil.clone())
                .wall(0.0, 0.0, 100.0, 0.0, 0, wall.clone())
                .wall(100.0, 0.0, 100.0, 50.0, 1, wall.clone())
                .portal(100.0, 50.0, 100.0, 70.0, 2, wall.clone(), wall.clone(), 0, 1)
                .wall(100.0, 70.0, 100.0, 100.0, 3, wall.clone())
                .wall(100.0, 100.0, 0.0, 100.0, 4, wall.clone())
                .wall(0.0, 100.0, 0.0, 0.0, 5, wall.clone())
                .build(),
            SectorBuilder::new(1, 0.0, 16.0, floor.clone(), ceil.clone())
                .wall(100.0, 50.0, 160.0, 50.0, 0, wall.clone())
                .wall(160.0, 50.0, 160.0, 70.0, 1, wall.clone())
                .wall(160.0, 70.0, 100.0, 70.0, 2, wall.clone())
                .portal(100.0, 70.0, 100.0, 50.0, 3, wall.clone(), wall.clone(), 1, 0)
                .build()
        ],
    }
}

/// Three sectors with different floor heights to simulate stairs.
pub fn stepped_rooms(asset_server: &AssetServer) -> Map {
    let wall = asset_server.load("texture.png");
    let floor = asset_server.load("floor_texture.png");
    let ceil = asset_server.load("floor_texture.png");

    let top = 20.0;

    Map {
        sectors: vec![
            // Bottom step — floor 0.0
            SectorBuilder::new(0, 0.0, top, floor.clone(), ceil.clone())
                .wall(0.0, 0.0, 80.0, 0.0, 0, wall.clone())
                .portal(80.0, 0.0, 80.0, 40.0, 1, wall.clone(), wall.clone(), 0, 1)
                .wall(80.0, 40.0, 0.0, 40.0, 2, wall.clone())
                .wall(0.0, 40.0, 0.0, 0.0, 3, wall.clone())
                .build(),
            // Middle step — floor 4.0
            SectorBuilder::new(1, 4.0, top, floor.clone(), ceil.clone())
                .wall(80.0, 0.0, 160.0, 0.0, 0, wall.clone())
                .portal(160.0, 0.0, 160.0, 40.0, 1, wall.clone(), wall.clone(), 1, 2)
                .wall(160.0, 40.0, 80.0, 40.0, 2, wall.clone())
                .portal(80.0, 40.0, 80.0, 0.0, 3, wall.clone(), wall.clone(), 1, 0)
                .build(),
            // Top step — floor 8.0
            SectorBuilder::new(2, 8.0, top, floor.clone(), ceil.clone())
                .wall(160.0, 0.0, 240.0, 0.0, 0, wall.clone())
                .wall(240.0, 0.0, 240.0, 40.0, 1, wall.clone())
                .wall(240.0, 40.0, 160.0, 40.0, 2, wall.clone())
                .portal(160.0, 40.0, 160.0, 0.0, 3, wall.clone(), wall.clone(), 2, 1)
                .build()
        ],
    }
}

/// A single large room with several box obstacles at various heights.
pub fn obstacle_course(asset_server: &AssetServer) -> Map {
    let wall = asset_server.load("texture.png");
    let floor = asset_server.load("floor_texture.png");
    let ceil = asset_server.load("floor_texture.png");
    let top_tex = asset_server.load("floor_texture.png");
    let bottom_tex = asset_server.load("floor_texture.png");

    Map {
        sectors: vec![
            SectorBuilder::new(0, 0.0, 24.0, floor.clone(), ceil.clone())
                .wall(0.0, 0.0, 200.0, 0.0, 0, wall.clone())
                .wall(200.0, 0.0, 200.0, 200.0, 1, wall.clone())
                .wall(200.0, 200.0, 0.0, 200.0, 2, wall.clone())
                .wall(0.0, 200.0, 0.0, 0.0, 3, wall.clone())
                // Tall pillar (floor to ceiling)
                .rect_obstacle(
                    0,
                    30.0,
                    30.0,
                    40.0,
                    40.0,
                    0.0,
                    24.0,
                    wall.clone(),
                    top_tex.clone(),
                    bottom_tex.clone()
                )
                // Short crate
                .rect_obstacle(
                    1,
                    60.0,
                    80.0,
                    70.0,
                    90.0,
                    0.0,
                    6.0,
                    wall.clone(),
                    top_tex.clone(),
                    bottom_tex.clone()
                )
                // Floating platform at eye level
                .rect_obstacle(
                    2,
                    100.0,
                    40.0,
                    120.0,
                    50.0,
                    2.0,
                    4.0,
                    wall.clone(),
                    top_tex.clone(),
                    bottom_tex.clone()
                )
                // Wide low wall
                .rect_obstacle(
                    3,
                    80.0,
                    140.0,
                    140.0,
                    146.0,
                    0.0,
                    3.0,
                    wall.clone(),
                    top_tex.clone(),
                    bottom_tex.clone()
                )
                // Pillar in the center
                .rect_obstacle(
                    4,
                    90.0,
                    90.0,
                    98.0,
                    98.0,
                    0.0,
                    24.0,
                    wall.clone(),
                    top_tex.clone(),
                    bottom_tex.clone()
                )
                .build()
        ],
    }
}

/// Non-rectangular sector: a pentagon-shaped room.
pub fn pentagon_room(asset_server: &AssetServer) -> Map {
    let wall = asset_server.load("texture.png");
    let floor = asset_server.load("floor_texture.png");
    let ceil = asset_server.load("floor_texture.png");

    // Centre at (100, 100), radius 80, 5 vertices
    let cx = 100.0;
    let cy = 100.0;
    let r = 80.0;
    let vertices: Vec<[f32; 2]> = (0..5)
        .map(|i| {
            let angle = (std::f32::consts::TAU * (i as f32)) / 5.0 - std::f32::consts::FRAC_PI_2;
            [cx + r * angle.cos(), cy + r * angle.sin()]
        })
        .collect();

    let mut builder = SectorBuilder::new(0, 0.0, 20.0, floor, ceil);
    for i in 0..5 {
        let next = (i + 1) % 5;
        builder = builder.wall(
            vertices[i][0],
            vertices[i][1],
            vertices[next][0],
            vertices[next][1],
            i,
            wall.clone()
        );
    }
    Map { sectors: vec![builder.build()] }
}

/// A two-storey room with a mezzanine ledge created by a stepped sector
/// and a pillar that goes through both floors.
pub fn mezzanine(asset_server: &AssetServer) -> Map {
    let wall = asset_server.load("texture.png");
    let floor = asset_server.load("floor_texture.png");
    let ceil = asset_server.load("floor_texture.png");
    let top_tex = asset_server.load("floor_texture.png");
    let bottom_tex = asset_server.load("floor_texture.png");

    Map {
        sectors: vec![
            // Main hall — full height, floor 0, ceiling 24
            SectorBuilder::new(0, 0.0, 24.0, floor.clone(), ceil.clone())
                .wall(0.0, 0.0, 120.0, 0.0, 0, wall.clone())
                .portal(120.0, 0.0, 120.0, 30.0, 1, wall.clone(), wall.clone(), 0, 1)
                .wall(120.0, 30.0, 120.0, 100.0, 2, wall.clone())
                .wall(120.0, 100.0, 80.0, 100.0, 3, wall.clone())
                .portal(80.0, 100.0, 60.0, 100.0, 4, wall.clone(), wall.clone(), 0, 1)
                .wall(60.0, 100.0, 0.0, 100.0, 5, wall.clone())
                .wall(0.0, 100.0, 0.0, 0.0, 6, wall.clone())
                .rect_obstacle(
                    0,
                    50.0,
                    30.0,
                    60.0,
                    50.0,
                    0.0,
                    24.0,
                    wall.clone(),
                    top_tex.clone(),
                    bottom_tex.clone()
                )
                .build(),
            // Mezzanine ledge — floor at 10, ceiling at 24
            SectorBuilder::new(1, 10.0, 24.0, floor.clone(), ceil.clone())
                .wall(120.0, 0.0, 140.0, 0.0, 0, wall.clone())
                .wall(140.0, 0.0, 140.0, 30.0, 1, wall.clone())
                .wall(140.0, 30.0, 120.0, 30.0, 2, wall.clone())
                .portal(120.0, 30.0, 120.0, 0.0, 3, wall.clone(), wall.clone(), 1, 0)
                .build()
        ],
    }
}

/// A map with two rooms side-by-side, each with different ceiling heights,
/// connected by a narrow corridor.
pub fn l_shape(asset_server: &AssetServer) -> Map {
    let wall = asset_server.load("texture.png");
    let floor = asset_server.load("floor_texture.png");
    let ceil = asset_server.load("floor_texture.png");

    Map {
        sectors: vec![
            SectorBuilder::new(0, 0.0, 20.0, floor.clone(), ceil.clone())
                .wall(0.0, 0.0, 100.0, 0.0, 0, wall.clone())
                .wall(100.0, 0.0, 100.0, 100.0, 1, wall.clone())
                .wall(100.0, 100.0, 0.0, 100.0, 2, wall.clone())
                .wall(0.0, 100.0, 0.0, 0.0, 3, wall.clone())
                .build(),
            SectorBuilder::new(1, 0.0, 12.0, floor.clone(), ceil.clone())
                .wall(140.0, 100.0, 240.0, 100.0, 0, wall.clone())
                .wall(240.0, 100.0, 240.0, 200.0, 1, wall.clone())
                .wall(240.0, 200.0, 140.0, 200.0, 2, wall.clone())
                .wall(140.0, 200.0, 140.0, 100.0, 3, wall.clone())
                .build(),
            SectorBuilder::new(2, 0.0, 20.0, floor.clone(), ceil.clone())
                .wall(100.0, 40.0, 100.0, 60.0, 0, wall.clone())
                .portal(100.0, 60.0, 100.0, 80.0, 1, wall.clone(), wall.clone(), 2, 3)
                .wall(100.0, 80.0, 100.0, 100.0, 2, wall.clone())
                .wall(100.0, 100.0, 140.0, 100.0, 3, wall.clone())
                .portal(120.0, 100.0, 140.0, 100.0, 4, wall.clone(), wall.clone(), 2, 1)
                .wall(140.0, 100.0, 140.0, 80.0, 5, wall.clone())
                .portal(140.0, 80.0, 140.0, 60.0, 6, wall.clone(), wall.clone(), 2, 3)
                .wall(140.0, 60.0, 140.0, 40.0, 7, wall.clone())
                .wall(140.0, 40.0, 100.0, 40.0, 8, wall.clone())
                .build(),
            SectorBuilder::new(3, 0.0, 8.0, floor.clone(), ceil.clone())
                .wall(100.0, 60.0, 140.0, 60.0, 0, wall.clone())
                .wall(140.0, 60.0, 140.0, 80.0, 1, wall.clone())
                .wall(140.0, 80.0, 100.0, 80.0, 2, wall.clone())
                .portal(100.0, 80.0, 100.0, 60.0, 3, wall.clone(), wall.clone(), 3, 2)
                .build()
        ],
    }
}

/// A cross-shaped map with a central hub and four wings, each with a different floor height.
pub fn cross_map(asset_server: &AssetServer) -> Map {
    let wall = asset_server.load("texture.png");
    let floor = asset_server.load("floor_texture.png");
    let ceil = asset_server.load("floor_texture.png");

    let hub = SectorBuilder::new(0, 0.0, 20.0, floor.clone(), ceil.clone())
        .wall(40.0, 40.0, 120.0, 40.0, 0, wall.clone())
        .portal(120.0, 40.0, 120.0, 60.0, 1, wall.clone(), wall.clone(), 0, 1)
        .wall(120.0, 60.0, 120.0, 120.0, 2, wall.clone())
        .portal(120.0, 120.0, 100.0, 120.0, 3, wall.clone(), wall.clone(), 0, 2)
        .wall(100.0, 120.0, 60.0, 120.0, 4, wall.clone())
        .portal(60.0, 120.0, 40.0, 120.0, 5, wall.clone(), wall.clone(), 0, 3)
        .wall(40.0, 120.0, 40.0, 60.0, 6, wall.clone())
        .portal(40.0, 60.0, 40.0, 40.0, 7, wall.clone(), wall.clone(), 0, 4)
        .wall(40.0, 40.0, 40.0, 40.0, 8, wall.clone())
        .build();

    let right_wing = SectorBuilder::new(1, 0.0, 20.0, floor.clone(), ceil.clone())
        .wall(120.0, 40.0, 200.0, 40.0, 0, wall.clone())
        .wall(200.0, 40.0, 200.0, 60.0, 1, wall.clone())
        .wall(200.0, 60.0, 120.0, 60.0, 2, wall.clone())
        .portal(120.0, 60.0, 120.0, 40.0, 3, wall.clone(), wall.clone(), 1, 0)
        .build();

    let bottom_wing = SectorBuilder::new(2, 4.0, 20.0, floor.clone(), ceil.clone())
        .wall(100.0, 120.0, 100.0, 200.0, 0, wall.clone())
        .wall(100.0, 200.0, 60.0, 200.0, 1, wall.clone())
        .wall(60.0, 200.0, 60.0, 120.0, 2, wall.clone())
        .portal(60.0, 120.0, 100.0, 120.0, 3, wall.clone(), wall.clone(), 2, 0)
        .build();

    let left_wing = SectorBuilder::new(3, 8.0, 20.0, floor.clone(), ceil.clone())
        .wall(40.0, 120.0, -40.0, 120.0, 0, wall.clone())
        .wall(-40.0, 120.0, -40.0, 100.0, 1, wall.clone())
        .wall(-40.0, 100.0, 40.0, 100.0, 2, wall.clone())
        .portal(40.0, 100.0, 40.0, 120.0, 3, wall.clone(), wall.clone(), 3, 0)
        .build();

    let top_wing = SectorBuilder::new(4, 12.0, 20.0, floor.clone(), ceil.clone())
        .wall(40.0, 40.0, 40.0, -40.0, 0, wall.clone())
        .wall(40.0, -40.0, 60.0, -40.0, 1, wall.clone())
        .wall(60.0, -40.0, 60.0, 40.0, 2, wall.clone())
        .portal(60.0, 40.0, 40.0, 40.0, 3, wall.clone(), wall.clone(), 4, 0)
        .build();

    Map { sectors: vec![hub, right_wing, bottom_wing, left_wing, top_wing] }
}

/// A tall column sector in the centre of a surrounding room, demonstrating
/// a "sector within a sector" pattern (the column is a separate sector whose
/// floor/ceiling are inside the outer sector's volume).
pub fn room_with_column(asset_server: &AssetServer) -> Map {
    let wall = asset_server.load("texture.png");
    let floor = asset_server.load("floor_texture.png");
    let ceil = asset_server.load("floor_texture.png");

    Map {
        sectors: vec![
            SectorBuilder::new(0, 0.0, 24.0, floor.clone(), ceil.clone())
                .wall(0.0, 0.0, 160.0, 0.0, 0, wall.clone())
                .wall(160.0, 0.0, 160.0, 160.0, 1, wall.clone())
                .wall(160.0, 160.0, 0.0, 160.0, 2, wall.clone())
                .wall(0.0, 160.0, 0.0, 0.0, 3, wall.clone())
                .build(),
            // Inner column — floor at 0, but walls are portals to outer sector
            SectorBuilder::new(1, 0.0, 24.0, floor.clone(), ceil.clone())
                .portal(70.0, 70.0, 90.0, 70.0, 0, wall.clone(), wall.clone(), 1, 0)
                .portal(90.0, 70.0, 90.0, 90.0, 1, wall.clone(), wall.clone(), 1, 0)
                .portal(90.0, 90.0, 70.0, 90.0, 2, wall.clone(), wall.clone(), 1, 0)
                .portal(70.0, 90.0, 70.0, 70.0, 3, wall.clone(), wall.clone(), 1, 0)
                .build()
        ],
    }
}

/// A sprial-shaped corridor that coils inwards.
/// Built entirely from walls (no portals) — a single sector.
pub fn spiral(asset_server: &AssetServer) -> Map {
    let wall = asset_server.load("texture.png");
    let floor = asset_server.load("floor_texture.png");
    let ceil = asset_server.load("floor_texture.png");

    let cx = 100.0;
    let cy = 100.0;
    let segments = 40;
    let turns = 3.0;
    let r_start = 80.0;
    let r_end = 10.0;

    let mut pts: Vec<[f32; 2]> = Vec::with_capacity(segments + 1);
    for i in 0..=segments {
        let t = (i as f32) / (segments as f32);
        let angle = turns * std::f32::consts::TAU * t;
        let r = r_start + (r_end - r_start) * t;
        pts.push([cx + r * angle.cos(), cy + r * angle.sin()]);
    }

    let mut builder = SectorBuilder::new(0, 0.0, 16.0, floor, ceil);
    for i in 0..segments {
        builder = builder.wall(pts[i][0], pts[i][1], pts[i + 1][0], pts[i + 1][1], i, wall.clone());
    }
    // Close the polygon
    builder = builder.wall(
        pts[segments][0],
        pts[segments][1],
        pts[0][0],
        pts[0][1],
        segments,
        wall.clone()
    );

    Map { sectors: vec![builder.build()] }
}

/// Recreation of the layout of DOOM E1M1 "Hangar".
/// Sectors are laid out following the original map's flow:
///   0  Start room (with the green slime pillar alcove)
///   1  Short corridor leading north
///   2  Zigzag hallway (the temperature-control corridor)
///   3  Computer room (raised control platform)
///   4  Dark room / aux storage
///   5  Exit room with the slime pit and the exit switch alcove
/// All coordinates are in fictional "Doom units" scaled to taste; the
/// topology mirrors the original level but the geometry is simplified
/// to axis-aligned sectors so it can be expressed with the existing
/// SectorBuilder / portal helpers.
pub fn doom_e1m1(asset_server: &AssetServer) -> Map {
    let wall: Handle<Image> = asset_server.load("texture.png");
    let floor: Handle<Image> = asset_server.load("floor_texture.png");
    let ceil: Handle<Image> = asset_server.load("floor_texture.png");
    let top_tex: Handle<Image> = asset_server.load("floor_texture.png");
    let bottom_tex: Handle<Image> = asset_server.load("floor_texture.png");

    // Common ceiling height for most rooms.
    let h = 16.0;
    // Slimepit floor in the exit room is lower.
    let slime_floor = -8.0;

    Map {
        sectors: vec![
            // 0 — Start room. The player begins here. A small alcove on the
            // west wall holds the slime pillar (an obstacle). The north wall
            // has a portal into the corridor.
            SectorBuilder::new(0, 0.0, h, floor.clone(), ceil.clone())
                .wall(0.0, 0.0, 96.0, 0.0, 0, wall.clone())
                .wall(96.0, 0.0, 96.0, 32.0, 1, wall.clone())
                .portal(96.0, 32.0, 96.0, 56.0, 2, wall.clone(), wall.clone(), 0, 1)
                .wall(96.0, 56.0, 96.0, 96.0, 3, wall.clone())
                .wall(96.0, 96.0, 0.0, 96.0, 4, wall.clone())
                .wall(0.0, 96.0, 0.0, 0.0, 5, wall.clone())
                // Slime pillar alcove obstacle (a raised block)
                .rect_obstacle(
                    0,
                    12.0,
                    40.0,
                    24.0,
                    56.0,
                    0.0,
                    10.0,
                    wall.clone(),
                    top_tex.clone(),
                    bottom_tex.clone()
                )
                .build(),

            // 1 — Short corridor connecting start room to zigzag hallway.
            SectorBuilder::new(1, 0.0, h, floor.clone(), ceil.clone())
                .wall(96.0, 32.0, 128.0, 32.0, 0, wall.clone())
                .wall(128.0, 32.0, 128.0, 56.0, 1, wall.clone())
                .portal(128.0, 56.0, 128.0, 32.0, 2, wall.clone(), wall.clone(), 1, 2)
                .portal(96.0, 56.0, 96.0, 32.0, 3, wall.clone(), wall.clone(), 1, 0)
                .build(),

            // 2 — Zigzag hallway. Two 90-degree turns, expressed as a single
            // L-shaped sector with internal wall segments forming the bends.
            SectorBuilder::new(2, 0.0, h, floor.clone(), ceil.clone())
                .wall(128.0, 32.0, 192.0, 32.0, 0, wall.clone())
                .wall(192.0, 32.0, 192.0, 8.0, 1, wall.clone())
                .wall(192.0, 8.0, 160.0, 8.0, 2, wall.clone())
                .wall(160.0, 8.0, 160.0, -32.0, 3, wall.clone())
                .wall(160.0, -32.0, 192.0, -32.0, 4, wall.clone())
                .wall(192.0, -32.0, 192.0, -56.0, 5, wall.clone())
                .portal(192.0, -56.0, 192.0, -32.0, 6, wall.clone(), wall.clone(), 2, 3)
                .wall(192.0, -32.0, 208.0, -32.0, 7, wall.clone())
                .portal(208.0, 8.0, 192.0, 8.0, 8, wall.clone(), wall.clone(), 2, 1)
                .wall(208.0, 32.0, 208.0, 8.0, 9, wall.clone())
                .build(),

            // 3 — Computer room. Raised control platform in the centre
            // (an obstacle the player can walk around). Floor is the same
            // height but the platform is elevated.
            SectorBuilder::new(3, 0.0, h, floor.clone(), ceil.clone())
                .wall(192.0, -56.0, 288.0, -56.0, 0, wall.clone())
                .wall(288.0, -56.0, 288.0, 8.0, 1, wall.clone())
                .wall(288.0, 8.0, 240.0, 8.0, 2, wall.clone())
                .portal(240.0, 8.0, 208.0, 8.0, 3, wall.clone(), wall.clone(), 3, 2)
                .wall(208.0, 8.0, 192.0, 8.0, 4, wall.clone())
                // Two-step raised computer platform
                .rect_obstacle(
                    0,
                    224.0,
                    -32.0,
                    256.0,
                    -16.0,
                    4.0,
                    h,
                    wall.clone(),
                    top_tex.clone(),
                    bottom_tex.clone()
                )
                .build(),

            // 4 — Dark aux room, branching off the computer room through a
            // narrow doorway. Lower ceiling to make it feel cramped.
            SectorBuilder::new(4, 0.0, 10.0, floor.clone(), ceil.clone())
                .wall(288.0, 8.0, 288.0, 24.0, 0, wall.clone())
                .wall(288.0, 24.0, 320.0, 24.0, 1, wall.clone())
                .wall(320.0, 24.0, 320.0, -32.0, 2, wall.clone())
                .wall(320.0, -32.0, 288.0, -32.0, 3, wall.clone())
                .wall(288.0, -32.0, 288.0, -16.0, 4, wall.clone())
                .portal(288.0, -16.0, 288.0, 8.0, 5, wall.clone(), wall.clone(), 4, 3)
                .build(),

            // 5 — Exit room. A drop into the slime pit (negative floor), the
            // exit switch alcove against the far wall (an obstacle serving
            // as the switch pedestal), and a portal back to the computer room.
            SectorBuilder::new(5, slime_floor, h, floor.clone(), ceil.clone())
                .portal(288.0, -56.0, 288.0, -32.0, 0, wall.clone(), wall.clone(), 5, 3)
                .wall(288.0, -32.0, 320.0, -32.0, 1, wall.clone())
                .wall(320.0, -32.0, 320.0, -120.0, 2, wall.clone())
                .wall(320.0, -120.0, 224.0, -120.0, 3, wall.clone())
                .wall(224.0, -120.0, 224.0, -56.0, 4, wall.clone())
                .wall(224.0, -56.0, 240.0, -56.0, 5, wall.clone())
                .wall(240.0, -56.0, 240.0, -72.0, 6, wall.clone())
                // Exit switch pedestal (raised block against the wall)
                .rect_obstacle(
                    0,
                    296.0,
                    -112.0,
                    312.0,
                    -104.0,
                    0.0,
                    14.0,
                    wall.clone(),
                    top_tex.clone(),
                    bottom_tex.clone()
                )
                .build(),
        ],
    }
}

/// A small "town square" with multiple buildings (obstacles) and
/// four surrounding sectors forming a cross.
pub fn town_square(asset_server: &AssetServer) -> Map {
    let wall: Handle<Image> = asset_server.load("texture.png");
    let floor: Handle<Image> = asset_server.load("floor_texture.png");
    let ceil: Handle<Image> = asset_server.load("floor_texture.png");

    Map {
        sectors: vec![
            // Central square
            SectorBuilder::new(0, 0.0, 20.0, floor.clone(), ceil.clone())
                .wall(40.0, 40.0, 120.0, 40.0, 0, wall.clone())
                .portal(120.0, 40.0, 120.0, 80.0, 1, wall.clone(), wall.clone(), 0, 1)
                .wall(120.0, 80.0, 120.0, 120.0, 2, wall.clone())
                .portal(120.0, 120.0, 80.0, 120.0, 3, wall.clone(), wall.clone(), 0, 2)
                .wall(80.0, 120.0, 40.0, 120.0, 4, wall.clone())
                .portal(40.0, 120.0, 40.0, 80.0, 5, wall.clone(), wall.clone(), 0, 3)
                .wall(40.0, 80.0, 40.0, 40.0, 6, wall.clone())
                .build(),
            // North building
            rect_sector(
                1,
                120.0,
                40.0,
                180.0,
                80.0,
                0.0,
                20.0,
                floor.clone(),
                wall.clone(),
                ceil.clone()
            ),
            // East building
            rect_sector(
                2,
                80.0,
                120.0,
                120.0,
                180.0,
                0.0,
                20.0,
                floor.clone(),
                wall.clone(),
                ceil.clone()
            ),
            // South building
            rect_sector(
                3,
                40.0,
                80.0,
                -20.0,
                120.0,
                0.0,
                20.0,
                floor.clone(),
                wall.clone(),
                ceil.clone()
            )
        ],
    }
}
