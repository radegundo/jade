use std::cmp::min;

use bevy::{ prelude::*, state::commands, transform };
use crate::{ systems::find_player_sector, * };
use map::*;
use ray::*;
use systems::*;

pub fn render_2d(
    mut gizmos: Gizmos<MapGizmos>,
    map: Res<Map>,
    view_info: Res<ViewInfo>,
    transform_query: Query<&Transform, With<Player>>
) {
    let transform = transform_query.single().unwrap();
    for i in 0..RAY_COUNT {
        if let Some(sector) = find_player_sector(transform.translation.truncate(), &map) {
            let sector = &map.sectors[sector];
            if let Some(hit) = get_hit_sector(&transform, &view_info, sector.id, &map, i) {
                let x = hit_to_screen_x(&view_info, i);
                let window_top = project_height(
                    map.sectors[sector.id].ceiling_height - EYE_OFFSET,
                    hit.perp_dist,
                    &view_info
                );
                let window_bottom = project_height(
                    map.sectors[sector.id].floor_height - EYE_OFFSET,
                    hit.perp_dist,
                    &view_info
                );
                gizmos.line_2d(Vec2::new(x, window_top), Vec2::new(x, window_bottom), Color::WHITE);
            }
        }
    }
}

pub fn render(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    player_cache: Res<PlayerCameraCache>,
    map: Res<Map>,
    view_info: Res<ViewInfo>,
    mut pool: ResMut<WallEntityPool>,
    mut query: Query<&mut Visibility>
) {
    let transform = &player_cache.transform;
    // Find the sector the player is in
    if let Some(player_sector_index) = find_player_sector(transform.translation.truncate(), &map) {
        let mut hits = Vec::new();
        //1. HIT GROUPING
        for i in 0..RAY_COUNT {
            //Get hits for the sector the player is in
            let hit = get_hit_sector(&transform, &view_info, player_sector_index, &map, i).unwrap();
            hits.push(hit);
        }
        let grouped_hits = group_hits_by_wall(hits);

        //2. POOL MANAGEMENT
        let needed = grouped_hits.len();
        let pool_size = pool.entities.len();
        //TODO GET ENTITY INFO
        for i in 0..min(needed, pool_size) {
            let entity = pool.entities[i];
            let hit_group = &grouped_hits[i];
            let sector = &map.sectors[hit_group[0].sector_id];
            let mesh = build_wall_mesh(hit_group, sector);
            commands
                .entity(entity)
                .insert(Visibility::Visible)
                .insert(Mesh3d(meshes.add(mesh)));
        }
        //3. HIDE NOT NEEDED ENTITIES
        for i in needed..pool.used.min(pool_size) {
            if let Ok(mut vis) = query.get_mut(pool.entities[i]) {
                *vis = Visibility::Hidden;
            }
        }

        //4. SPAWN OVERFLOW
        if needed > pool_size {
            for i in pool_size..needed {
                let hit_group = &grouped_hits[i];
                let sector = &map.sectors[hit_group[0].sector_id];

                let entity = commands
                    .spawn((
                        Visibility::Visible,
                        Mesh2d(meshes.add(build_wall_mesh(hit_group, sector))),
                        // MeshMaterial2d(materials.add(build_wall_material(hit_group, sector))),
                        Transform::default(),
                    ))
                    .id();

                pool.entities.push(entity);
            }
        }
    }
}

// pub fn mesh_setup(
//     mut commands: Commands,
//     map: Res<Map>,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<StandardMaterial>>
// ) {
//     for sector in &map.sectors {
//         for wall in &sector.walls {
//             if wall.back_side_def.is_some() {
//                 continue; // Skip back-facing walls for now
//             }
//             let mesh = build_wall_mesh(wall, &sector);
//             let material = StandardMaterial {
//                 base_color_texture: Some(
//                     wall.front_side_def.textures.middle.clone().unwrap().clone()
//                 ),
//                 ..default()
//             };
//             commands.spawn((
//                 Mesh3d(meshes.add(mesh)),
//                 MeshMaterial3d(materials.add(material)),
//                 Transform::default(),
//             ));
//         }
//     }
// }

// ------------------------------RENDER HELPERS------------------------------
fn project_height(world_height: f32, dist: f32, view_info: &ViewInfo) -> f32 {
    let relative = world_height - view_info.eye_height;
    (relative * view_info.view_distance) / dist + view_info.pitch
}

pub fn build_wall_mesh(hit_group: &Vec<WallHit>, sector: &Sector) -> Mesh {
    // TODO: Implement mesh building logic based on the hit group and sector information
    let mesh = Mesh::new(bevy::mesh::PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    mesh
}

// pub fn build_wall_mesh(wall: &LineDef, sector: &Sector) -> Mesh {
//     let normal = wall_normal(wall).extend(0.0);
//     let mesh = Mesh::new(bevy::mesh::PrimitiveTopology::TriangleList, RenderAssetUsages::default())
//         .with_inserted_attribute(
//             Mesh::ATTRIBUTE_POSITION,
//             vec![
//                 [wall.start.x, wall.start.y, sector.floor_height],
//                 [wall.end.x, wall.end.y, sector.floor_height],
//                 [wall.end.x, wall.end.y, sector.ceiling_height],
//                 [wall.start.x, wall.start.y, sector.ceiling_height]
//             ]
//         )
//         .with_inserted_attribute(
//             Mesh::ATTRIBUTE_NORMAL,
//             vec![
//                 [normal.x, normal.y, normal.z],
//                 [normal.x, normal.y, normal.z],
//                 [normal.x, normal.y, normal.z],
//                 [normal.x, normal.y, normal.z]
//             ]
//         )
//         .with_inserted_attribute(
//             Mesh::ATTRIBUTE_UV_0,
//             vec![[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]]
//         )
//         .with_inserted_indices(Indices::U32(vec![0, 2, 1, 0, 3, 2]));
//     mesh
// }
fn wall_normal(line_def: &LineDef) -> Vec2 {
    let dir = (line_def.end - line_def.start).normalize_or_zero();
    Vec2::new(dir.y, -dir.x) // inward normal for CCW sectors
}
