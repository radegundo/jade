// use bevy::prelude::*;

// use crate::*;
// use ray::*;
// use sprite::*;

// const TILE_SIZE: f32 = 64.0;

// #[derive(Resource)]
// pub struct Light {
//     pub direction: Vec3,
//     pub color: Color,
//     pub intensity: f32,
// }

// pub fn render(
//     mut commands: Commands,
//     mut hits: ResMut<Hits>,
//     view_info: Res<ViewInfo>,
//     player_cache: Res<PlayerCameraCache>,
//     map: Res<Map>,
//     light: Res<Light>,
//     mut clip: ResMut<Vclip>,
//     mut pool: ResMut<SpritePool>,
//     mut sprite_query: Query<(&mut Sprite, &mut Transform, &mut Visibility)>
// ) {
//     pool.reset();
//     let player_pos = player_cache.transform.translation.truncate();
//     if let Some(i) = find_player_sector(player_pos, &map) {
//         get_sector_hits(&player_cache, &mut hits, i, &map, &view_info);
//     } else {
//         for hit in hits.hits.iter_mut() {
//             *hit = None;
//         }
//     }

//     for i in 0..RAY_COUNT {
//         if let Some(hit) = &hits.hits[i] {
//             clip.0[i] = VBounds::full();
//             render_wall_collumn(
//                 i,
//                 &player_cache,
//                 total_dist,
//                 hit.perp_dist,
//                 clip,
//                 view_info,
//                 gizmos,
//                 light,
//                 map,
//                 &mut commands,
//                 pool
//             );
//         }
//         pool.hide_unused(&mut sprite_query);
//     }
// }
// pub fn get_relative_coords(transform: &Transform, coords: Vec2) -> Vec2 {
//     let dx = coords.x - transform.translation.x;
//     let dy = coords.y - transform.translation.y;

//     let angle = transform.rotation.to_euler(EulerRot::YXZ).0;
//     let rel_x = dx * angle.cos() + dy * angle.sin();
//     let rel_y = -dx * angle.sin() + dy * angle.cos();

//     Vec2::new(rel_x, rel_y)
// }

// pub fn update_eye_height(
//     player_cache: Res<PlayerCameraCache>,
//     map: Res<Map>,
//     mut view_info: ResMut<ViewInfo>,
//     time: Res<Time>
// ) {
//     let pos = player_cache.transform.translation.truncate();

//     if let Some(sector_idx) = find_player_sector(pos, &map) {
//         let sector = &map.sectors[sector_idx];
//         let target_eye_height = sector.floor_height + EYE_OFFSET;

//         let speed = 8.0; // higher = snappier transition
//         view_info.eye_height =
//             view_info.eye_height +
//             (target_eye_height - view_info.eye_height) * (speed * time.delta_secs()).min(1.0);
//     }
// }

// fn project_height(world_height: f32, dist: f32, view_info: &ViewInfo) -> f32 {
//     let relative = world_height - view_info.eye_height;
//     (relative * view_info.view_distance) / dist + view_info.pitch
// }

// // pub fn render_column(
// //     index: usize,
// //     player_cache: &PlayerCameraCache,
// //     total_dist: f32,
// //     hit: &WallHit,
// //     mut clip: VBounds,
// //     view_info: &ViewInfo,
// //     gizmos: &mut Gizmos,
// //     light: &Light,
// //     map: &Map
// // ) {
// //     let x = hit_to_screen_x(view_info, index);
// //     let sector = if let SectorType::ObstacleSector = hit.sector_type {
// //         &map.obstacle_sectors[hit.sector_id]
// //     } else {
// //         &map.sectors[hit.sector_id]
// //     };

// //     match &hit.line_def.back_side_def {
// //         None => {
// //             let wall_top_screen = project_height(
// //                 sector.ceiling_height,
// //                 total_dist,
// //                 view_info
// //             ).clamp(clip.bottom, clip.top);
// //             let wall_bottom_screen = project_height(
// //                 sector.floor_height,
// //                 total_dist,
// //                 view_info
// //             ).clamp(clip.bottom, clip.top);

// //             //OBSTACLE CODE FOR LATER
// //             // if let SectorType::ObstacleSector = hit.sector_type {
// //             //     // --- Draw the obstacle's front face ---
// //             //     let color = shade_color_directional(
// //             //         hit.line_def.front_side_def.middle_texture.unwrap(),
// //             //         wall_normal(&hit.line_def),
// //             //         total_dist,
// //             //         view_info,
// //             //         light
// //             //     );
// //             //     let room = &map.sectors[hit.room_sector_id]; // the containing room
// //             //     gizmos.line_2d(
// //             //         Vec2::new(x, wall_top_screen),
// //             //         Vec2::new(x, wall_bottom_screen),
// //             //         color
// //             //     );

// //             //     // --- Shrink clip around the obstacle's screen footprint ---
// //             //     let mut behind_clip = clip;
// //             //     behind_clip.bottom = wall_top_screen; // nothing behind can draw below the obstacle's top
// //             //     // (if the obstacle doesn't reach the floor, you'd also clip.top = wall_bottom_screen
// //             //     //  for a second segment below it — skipping that for now assuming floor-to-top boxes)

// //             //     // --- Continue the SAME ray, in the room sector the obstacle sits in, past it ---
// //             //     let angle = get_ray_angle(index, &player_cache.transform, view_info);
// //             //     let dir = Vec2::new(angle.cos(), angle.sin());
// //             //     let nudged_origin = hit.pos + dir * 0.05;
// //             //     let mut nudged_transform = player_cache.transform.clone();
// //             //     nudged_transform.translation = nudged_origin.extend(0.0);

// //             //     // Note: pass the room's sector_index the obstacle belongs to, not hit.sector_id
// //             //     // (hit.sector_id is the obstacle's own index — you need the containing room's index here)
// //             //     if
// //             //         let Some(next_hit) = get_single_hit(
// //             //             &nudged_transform,
// //             //             view_info,
// //             //             sector.id, // see note below — you need this on WallHit
// //             //             map,
// //             //             index
// //             //         )
// //             //     {
// //             //         let next_total_dist = total_dist + next_hit.perp_dist;
// //             //         render_column(
// //             //             index,
// //             //             player_cache,
// //             //             next_total_dist,
// //             //             &next_hit,
// //             //             behind_clip,
// //             //             view_info,
// //             //             gizmos,
// //             //             light,
// //             //             map
// //             //         );
// //             //     }

// //             //     draw_floor(
// //             //         x,
// //             //         wall_bottom_screen,
// //             //         clip.bottom,
// //             //         room.floor_texture,
// //             //         total_dist,
// //             //         view_info,
// //             //         light,
// //             //         gizmos
// //             //     );
// //             //     // no draw_ceiling here yet — see top-face section below
// //             //     return; // don't fall through to normal solid-wall floor/ceiling draw
// //             // }

// //             // let color = shade_color_directional(
// //             //     hit.line_def.front_side_def.middle_texture.unwrap(),
// //             //     wall_normal(&hit.line_def),
// //             //     total_dist,
// //             //     &view_info,
// //             //     light
// //             // );
// //             // Solid wall —> draw within the clip window, then stop.
// //             // gizmos.line_2d(Vec2::new(x, wall_top_screen), Vec2::new(x, wall_bottom_screen), color);
// //             draw_wall_column(
// //                 commands,
// //                 pool,
// //                 sprite_query,
// //                 x,
// //                 wall_bottom_screen,
// //                 wall_bottom_screen,
// //                 wall_u(hit.pos, &hit.line_def),
// //                 hit.line_def.front_side_def.middle_texture.clone().unwrap(),
// //                 tex_size,
// //                 dist,
// //                 normal,
// //                 view_info,
// //                 light
// //             );

// //             draw_floor(
// //                 x,
// //                 wall_bottom_screen,
// //                 clip.bottom,
// //                 sector.floor_texture,
// //                 total_dist,
// //                 &view_info,
// //                 &light,
// //                 gizmos
// //             );
// //             draw_ceiling(
// //                 x,
// //                 wall_top_screen,
// //                 clip.top,
// //                 sector.ceiling_texture,
// //                 total_dist,
// //                 &view_info,
// //                 &light,
// //                 gizmos
// //             );
// //         }
// //         Some(back) => {
// //             let wall_top_screen = project_height(
// //                 sector.ceiling_height,
// //                 total_dist,
// //                 view_info
// //             ).clamp(clip.bottom, clip.top);
// //             let wall_bottom_screen = project_height(
// //                 sector.floor_height,
// //                 total_dist,
// //                 view_info
// //             ).clamp(clip.bottom, clip.top);

// //             let back_sector = &map.sectors[back.sector];

// //             draw_floor(
// //                 x,
// //                 wall_bottom_screen,
// //                 clip.bottom,
// //                 sector.floor_texture,
// //                 total_dist,
// //                 &view_info,
// //                 &light,
// //                 gizmos
// //             );
// //             draw_ceiling(
// //                 x,
// //                 wall_top_screen,
// //                 clip.top,
// //                 sector.ceiling_texture,
// //                 total_dist,
// //                 &view_info,
// //                 &light,
// //                 gizmos
// //             );

// //             // Upper step (lowered ceiling ahead)
// //             if back_sector.ceiling_height < sector.ceiling_height {
// //                 let upper_bottom = project_height(
// //                     back_sector.ceiling_height,
// //                     total_dist,
// //                     view_info
// //                 ).clamp(clip.bottom, clip.top);
// //                 if let Some(color) = hit.line_def.front_side_def.upper_texture {
// //                     let color = shade_color_directional(
// //                         color,
// //                         wall_normal(&hit.line_def),
// //                         total_dist,
// //                         view_info,
// //                         light
// //                     );
// //                     gizmos.line_2d(
// //                         Vec2::new(x, wall_top_screen),
// //                         Vec2::new(x, upper_bottom),
// //                         color
// //                     );
// //                 }
// //                 clip.top = upper_bottom; // <-- SHRINK the window: nothing beyond can draw above this
// //             }

// //             // Lower step (raised floor ahead)
// //             if back_sector.floor_height > sector.floor_height {
// //                 let lower_top = project_height(
// //                     back_sector.floor_height,
// //                     total_dist,
// //                     view_info
// //                 ).clamp(clip.bottom, clip.top);
// //                 if let Some(color) = hit.line_def.front_side_def.lower_texture {
// //                     let color = shade_color_directional(
// //                         color,
// //                         wall_normal(&hit.line_def),
// //                         total_dist,
// //                         view_info,
// //                         light
// //                     );
// //                     gizmos.line_2d(
// //                         Vec2::new(x, lower_top),
// //                         Vec2::new(x, wall_bottom_screen),
// //                         color
// //                     );
// //                 }
// //                 clip.bottom = lower_top; // <-- SHRINK the window: nothing beyond can draw below this
// //             }
// //             // Lower floor ahead
// //             if back_sector.floor_height < sector.floor_height {
// //                 let lower_top = project_height(sector.floor_height, total_dist, view_info).clamp(
// //                     clip.bottom,
// //                     clip.top
// //                 );
// //                 clip.bottom = lower_top;
// //             }
// //             if back_sector.ceiling_height > sector.ceiling_height {
// //                 let upper_bottom = project_height(
// //                     sector.ceiling_height,
// //                     total_dist,
// //                     view_info
// //                 ).clamp(clip.bottom, clip.top);
// //                 clip.top = upper_bottom;
// //             }

// //             // Step into the next sector, carrying the SHRUNKEN clip forward
// //             let angle = get_ray_angle(index, &player_cache.transform, view_info);
// //             let dir = Vec2::new(angle.cos(), angle.sin());
// //             let nudged_origin = hit.pos + dir * 0.05;
// //             let mut nudged_transform = player_cache.transform.clone();
// //             nudged_transform.translation = nudged_origin.extend(0.0);

// //             if
// //                 let Some(next_hit) = get_single_hit(
// //                     &nudged_transform,
// //                     view_info,
// //                     back.sector,
// //                     &map,
// //                     index
// //                 )
// //             {
// //                 let next_total_dist = total_dist + next_hit.perp_dist;
// //                 render_column(
// //                     index,
// //                     player_cache,
// //                     next_total_dist,
// //                     &next_hit,
// //                     clip,
// //                     view_info,
// //                     gizmos,
// //                     &light,
// //                     map
// //                 );
// //             }
// //         }
// //     }
// // }
// pub fn render_wall_collumn(
//     index: usize,
//     player_cache: &PlayerCameraCache,
//     total_dist: f32,
//     hit: &WallHit,
//     mut clip: VBounds,
//     view_info: &ViewInfo,
//     gizmos: &mut Gizmos,
//     light: &Light,
//     map: &Map,
//     commands: &mut Commands,
//     pool: &mut SpritePool
// ) {
//     let x = hit_to_screen_x(view_info, index);
//     let sector = if let SectorType::ObstacleSector = hit.sector_type {
//         &map.obstacle_sectors[hit.sector_id]
//     } else {
//         &map.sectors[hit.sector_id]
//     };

//     match &hit.line_def.back_side_def {
//         None => {
//             let wall_top_screen = project_height(
//                 sector.ceiling_height,
//                 total_dist,
//                 view_info
//             ).clamp(clip.bottom, clip.top);
//             let wall_bottom_screen = project_height(
//                 sector.floor_height,
//                 total_dist,
//                 view_info
//             ).clamp(clip.bottom, clip.top);

//             draw_wall_column(
//                 commands,
//                 pool,
//                 sprite_query,
//                 x,
//                 wall_bottom_screen,
//                 wall_bottom_screen,
//                 wall_u(hit.pos, &hit.line_def),
//                 hit.line_def.front_side_def.middle_texture.clone().unwrap(),
//                 Vec2::new(64.0, 64.0),
//                 total_dist,
//                 wall_normal(&hit.line_def),
//                 view_info,
//                 light
//             );

//             draw_floor(
//                 x,
//                 wall_bottom_screen,
//                 clip.bottom,
//                 sector.floor_texture,
//                 total_dist,
//                 &view_info,
//                 &light,
//                 gizmos
//             );
//             draw_ceiling(
//                 x,
//                 wall_top_screen,
//                 clip.top,
//                 sector.ceiling_texture,
//                 total_dist,
//                 &view_info,
//                 &light,
//                 gizmos
//             );
//         }
//         Some(back) => {
//             let wall_top_screen = project_height(
//                 sector.ceiling_height,
//                 total_dist,
//                 view_info
//             ).clamp(clip.bottom, clip.top);
//             let wall_bottom_screen = project_height(
//                 sector.floor_height,
//                 total_dist,
//                 view_info
//             ).clamp(clip.bottom, clip.top);

//             let back_sector = &map.sectors[back.sector];

//             draw_floor(
//                 x,
//                 wall_bottom_screen,
//                 clip.bottom,
//                 sector.floor_texture,
//                 total_dist,
//                 &view_info,
//                 &light,
//                 gizmos
//             );
//             draw_ceiling(
//                 x,
//                 wall_top_screen,
//                 clip.top,
//                 sector.ceiling_texture,
//                 total_dist,
//                 &view_info,
//                 &light,
//                 gizmos
//             );

//             // Upper step (lowered ceiling ahead)
//             if back_sector.ceiling_height < sector.ceiling_height {
//                 let upper_bottom = project_height(
//                     back_sector.ceiling_height,
//                     total_dist,
//                     view_info
//                 ).clamp(clip.bottom, clip.top);
//                 if let Some(color) = hit.line_def.front_side_def.upper_texture {
//                     let color = shade_color_directional(
//                         color,
//                         wall_normal(&hit.line_def),
//                         total_dist,
//                         view_info,
//                         light
//                     );
//                     gizmos.line_2d(
//                         Vec2::new(x, wall_top_screen),
//                         Vec2::new(x, upper_bottom),
//                         color
//                     );
//                 }
//                 clip.top = upper_bottom; // <-- SHRINK the window: nothing beyond can draw above this
//             }

//             // Lower step (raised floor ahead)
//             if back_sector.floor_height > sector.floor_height {
//                 let lower_top = project_height(
//                     back_sector.floor_height,
//                     total_dist,
//                     view_info
//                 ).clamp(clip.bottom, clip.top);
//                 if let Some(color) = hit.line_def.front_side_def.lower_texture {
//                     let color = shade_color_directional(
//                         color,
//                         wall_normal(&hit.line_def),
//                         total_dist,
//                         view_info,
//                         light
//                     );
//                     gizmos.line_2d(
//                         Vec2::new(x, lower_top),
//                         Vec2::new(x, wall_bottom_screen),
//                         color
//                     );
//                 }
//                 clip.bottom = lower_top; // <-- SHRINK the window: nothing beyond can draw below this
//             }
//             // Lower floor ahead
//             if back_sector.floor_height < sector.floor_height {
//                 let lower_top = project_height(sector.floor_height, total_dist, view_info).clamp(
//                     clip.bottom,
//                     clip.top
//                 );
//                 clip.bottom = lower_top;
//             }
//             if back_sector.ceiling_height > sector.ceiling_height {
//                 let upper_bottom = project_height(
//                     sector.ceiling_height,
//                     total_dist,
//                     view_info
//                 ).clamp(clip.bottom, clip.top);
//                 clip.top = upper_bottom;
//             }

//             // Step into the next sector, carrying the SHRUNKEN clip forward
//             let angle = get_ray_angle(index, &player_cache.transform, view_info);
//             let dir = Vec2::new(angle.cos(), angle.sin());
//             let nudged_origin = hit.pos + dir * 0.05;
//             let mut nudged_transform = player_cache.transform.clone();
//             nudged_transform.translation = nudged_origin.extend(0.0);

//             if
//                 let Some(next_hit) = get_single_hit(
//                     &nudged_transform,
//                     view_info,
//                     back.sector,
//                     &map,
//                     index
//                 )
//             {
//                 let next_total_dist = total_dist + next_hit.perp_dist;
//                 render_column(
//                     index,
//                     player_cache,
//                     next_total_dist,
//                     &next_hit,
//                     clip,
//                     view_info,
//                     gizmos,
//                     &light,
//                     map
//                 );
//             }
//         }
//     }
// }

// fn draw_floor(
//     x: f32,
//     wall_bottom: f32,
//     clip_bottom: f32,
//     base_color: Color,
//     dist: f32,
//     view_info: &ViewInfo,
//     light: &Light,
//     gizmos: &mut Gizmos
// ) {
//     if wall_bottom > clip_bottom {
//         let color = shade_color_directional(base_color, FLOOR_NORMAL, dist, view_info, light);
//         gizmos.line_2d(Vec2::new(x, wall_bottom), Vec2::new(x, clip_bottom), color);
//     }
// }

// fn draw_ceiling(
//     x: f32,
//     wall_top: f32,
//     clip_top: f32,
//     base_color: Color,
//     dist: f32,
//     view_info: &ViewInfo,
//     light: &Light,
//     gizmos: &mut Gizmos
// ) {
//     if wall_top < clip_top {
//         let color = shade_color_directional(base_color, CEILING_NORMAL, dist, view_info, light);
//         gizmos.line_2d(Vec2::new(x, wall_top), Vec2::new(x, clip_top), color);
//     }
// }

// pub fn shade_color_directional(
//     color: Color,
//     normal: Vec3,
//     dist: f32,
//     view_info: &ViewInfo,
//     light: &Light
// ) -> Color {
//     let light_dir = light.direction.normalize();
//     let ndotl = normal.normalize().dot(-light_dir).max(0.0);

//     let ambient = 0.8;
//     let diffuse = ndotl * light.intensity;
//     let directional_brightness = (ambient + diffuse).min(1.0);

//     let max_dist = view_info.max_distance;
//     let t = (dist / max_dist).clamp(0.0, 1.0);
//     let dist_falloff = 1.0 - t * 0.7;

//     let brightness = directional_brightness * dist_falloff;

//     let srgba = color.to_srgba();
//     let light_srgba = light.color.to_srgba();
//     Color::srgba(
//         srgba.red * brightness * light_srgba.red,
//         srgba.green * brightness * light_srgba.green,
//         srgba.blue * brightness * light_srgba.blue,
//         srgba.alpha
//     )
// }

// //TURN BRIGHTNESS SCALAR TO TEXTURE TINT
// pub fn shading_tint(normal: Vec3, dist: f32, view_info: &ViewInfo, light: &Light) -> Color {
//     let brightness = shading_brightness(normal, dist, view_info, light);
//     let light_srgba = light.color.to_srgba();
//     Color::srgba(
//         brightness * light_srgba.red,
//         brightness * light_srgba.green,
//         brightness * light_srgba.blue,
//         1.0
//     )
// }
// pub fn shading_brightness(normal: Vec3, dist: f32, view_info: &ViewInfo, light: &Light) -> f32 {
//     let light_dir = light.direction.normalize();
//     let ndotl = normal.normalize().dot(-light_dir).max(0.0);

//     let ambient = 0.8;
//     let diffuse = ndotl * light.intensity;
//     let directional_brightness = (ambient + diffuse).min(1.0);

//     let t = (dist / view_info.max_distance).clamp(0.0, 1.0);
//     let dist_falloff = 1.0 - t * 0.7;

//     directional_brightness * dist_falloff
// }

// pub fn wall_normal(line_def: &LineDef) -> Vec3 {
//     let dir = (line_def.end - line_def.start).normalize();
//     Vec2::new(dir.y, -dir.x).extend(0.0)
// }

// fn draw_wall_column(
//     commands: &mut Commands,
//     pool: &mut SpritePool,
//     sprite_query: &mut Query<(&mut Sprite, &mut Transform, &mut Visibility)>,
//     x: f32,
//     wall_top: f32,
//     wall_bottom: f32,
//     u: f32, // fractional position along the wall — from wall_u()
//     texture: Handle<Image>,
//     tex_size: Vec2, // the texture's actual pixel dimensions, e.g. Vec2::new(64.0, 64.0)
//     dist: f32,
//     normal: Vec3,
//     view_info: &ViewInfo,
//     light: &Light
// ) {
//     let tiled_u = u.fract(); // wrap into 0.0..1.0 so the texture repeats along the wall
//     let tint = shading_tint(normal, dist, view_info, light);

//     let entity = pool.next(commands);
//     let height = (wall_top - wall_bottom).abs().max(1.0);
//     let center_y = (wall_top + wall_bottom) / 2.0;

//     if let Ok((mut sprite, mut transform, mut vis)) = sprite_query.get_mut(entity) {
//         sprite.image = texture;
//         sprite.color = tint;
//         sprite.custom_size = Some(Vec2::new(1.0, height));
//         sprite.rect = Some(
//             Rect::new(
//                 tiled_u * tex_size.x,
//                 0.0,
//                 tiled_u * tex_size.x + 1.0, // roughly one texel wide
//                 tex_size.y
//             )
//         );
//         transform.translation = Vec3::new(x, center_y, 0.0);
//         *vis = Visibility::Visible;
//     }
// }

// fn wall_u(world_pos: Vec2, line_def: &LineDef) -> f32 {
//     (world_pos - line_def.start).length() / TILE_SIZE
// }

use bevy::prelude::*;

use crate::*;
use ray::*;
use sprite::*;

const TILE_SIZE: f32 = 64.0;
const DEFAULT_TEX_SIZE: Vec2 = Vec2::new(4096.0, 4096.0);

#[derive(Resource)]
pub struct Light {
    pub direction: Vec3,
    pub color: Color,
    pub intensity: f32,
}

pub fn render(
    mut commands: Commands,
    mut hits: ResMut<Hits>,
    view_info: Res<ViewInfo>,
    player_cache: Res<PlayerCameraCache>,
    map: Res<Map>,
    light: Res<Light>,
    mut clip: ResMut<Vclip>,
    mut pool: ResMut<SpritePool>,
    mut sprite_query: Query<(&mut Sprite, &mut Transform, &mut Visibility)>
) {
    pool.reset();

    let player_pos = player_cache.transform.translation.truncate();
    if let Some(i) = find_player_sector(player_pos, &map) {
        get_sector_hits(&player_cache, &mut hits, i, &map, &view_info);
    } else {
        for hit in hits.hits.iter_mut() {
            *hit = None;
        }
    }

    for i in 0..RAY_COUNT {
        if let Some(hit) = &hits.hits[i] {
            clip.0[i] = VBounds::full();
            render_wall_column(
                i,
                &player_cache,
                hit.perp_dist,
                hit,
                clip.0[i],
                &view_info,
                &light,
                &map,
                &mut commands,
                &mut pool,
                &mut sprite_query
            );
        }
    }

    pool.hide_unused(&mut sprite_query);
}

pub fn get_relative_coords(transform: &Transform, coords: Vec2) -> Vec2 {
    let dx = coords.x - transform.translation.x;
    let dy = coords.y - transform.translation.y;

    let angle = transform.rotation.to_euler(EulerRot::YXZ).0;
    let rel_x = dx * angle.cos() + dy * angle.sin();
    let rel_y = -dx * angle.sin() + dy * angle.cos();

    Vec2::new(rel_x, rel_y)
}

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

fn project_height(world_height: f32, dist: f32, view_info: &ViewInfo) -> f32 {
    let relative = world_height - view_info.eye_height;
    (relative * view_info.view_distance) / dist + view_info.pitch
}

pub fn render_wall_column(
    index: usize,
    player_cache: &PlayerCameraCache,
    total_dist: f32,
    hit: &WallHit,
    mut clip: VBounds,
    view_info: &ViewInfo,
    light: &Light,
    map: &Map,
    commands: &mut Commands,
    pool: &mut SpritePool,
    sprite_query: &mut Query<(&mut Sprite, &mut Transform, &mut Visibility)>
) {
    let x = hit_to_screen_x(view_info, index);
    let sector = if let SectorType::ObstacleSector = hit.sector_type {
        &map.obstacle_sectors[hit.sector_id]
    } else {
        &map.sectors[hit.sector_id]
    };

    match &hit.line_def.back_side_def {
        None => {
            let wall_top_screen = project_height(
                sector.ceiling_height,
                total_dist,
                view_info
            ).clamp(clip.bottom, clip.top);
            let wall_bottom_screen = project_height(
                sector.floor_height,
                total_dist,
                view_info
            ).clamp(clip.bottom, clip.top);

            draw_wall_column(
                commands,
                pool,
                sprite_query,
                index,
                x,
                wall_top_screen, // was wall_bottom_screen — fixed
                wall_bottom_screen,
                wall_u(hit.pos, &hit.line_def),
                hit.line_def.front_side_def.middle_texture.clone().unwrap(),
                DEFAULT_TEX_SIZE,
                total_dist,
                wall_normal(&hit.line_def),
                view_info,
                light
            );

            draw_floor(
                commands,
                pool,
                sprite_query,
                index,
                x,
                wall_bottom_screen,
                clip.bottom,
                sector.floor_texture.clone(),
                DEFAULT_TEX_SIZE,
                total_dist,
                view_info,
                light
            );
            draw_ceiling(
                commands,
                pool,
                sprite_query,
                index,
                x,
                wall_top_screen,
                clip.top,
                sector.ceiling_texture.clone(),
                DEFAULT_TEX_SIZE,
                total_dist,
                view_info,
                light
            );
        }
        Some(back) => {
            let wall_top_screen = project_height(
                sector.ceiling_height,
                total_dist,
                view_info
            ).clamp(clip.bottom, clip.top);
            let wall_bottom_screen = project_height(
                sector.floor_height,
                total_dist,
                view_info
            ).clamp(clip.bottom, clip.top);

            let back_sector = &map.sectors[back.sector];

            draw_floor(
                commands,
                pool,
                sprite_query,
                index,
                x,
                wall_bottom_screen,
                clip.bottom,
                sector.floor_texture.clone(),
                DEFAULT_TEX_SIZE,
                total_dist,
                view_info,
                light
            );
            draw_ceiling(
                commands,
                pool,
                sprite_query,
                index,
                x,
                wall_top_screen,
                clip.top,
                sector.ceiling_texture.clone(),
                DEFAULT_TEX_SIZE,
                total_dist,
                view_info,
                light
            );

            // Upper step (lowered ceiling ahead)
            if back_sector.ceiling_height < sector.ceiling_height {
                let upper_bottom = project_height(
                    back_sector.ceiling_height,
                    total_dist,
                    view_info
                ).clamp(clip.bottom, clip.top);
                if let Some(texture) = &hit.line_def.front_side_def.upper_texture {
                    draw_wall_column(
                        commands,
                        pool,
                        sprite_query,
                        index,
                        x,
                        wall_top_screen,
                        upper_bottom,
                        wall_u(hit.pos, &hit.line_def),
                        texture.clone(),
                        DEFAULT_TEX_SIZE,
                        total_dist,
                        wall_normal(&hit.line_def),
                        view_info,
                        light
                    );
                }
                clip.top = upper_bottom;
            }

            // Lower step (raised floor ahead)
            if back_sector.floor_height > sector.floor_height {
                let lower_top = project_height(
                    back_sector.floor_height,
                    total_dist,
                    view_info
                ).clamp(clip.bottom, clip.top);
                if let Some(texture) = &hit.line_def.front_side_def.lower_texture {
                    draw_wall_column(
                        commands,
                        pool,
                        sprite_query,
                        index,
                        x,
                        lower_top,
                        wall_bottom_screen,
                        wall_u(hit.pos, &hit.line_def),
                        texture.clone(),
                        DEFAULT_TEX_SIZE,
                        total_dist,
                        wall_normal(&hit.line_def),
                        view_info,
                        light
                    );
                }
                clip.bottom = lower_top;
            }

            if back_sector.floor_height < sector.floor_height {
                let lower_top = project_height(sector.floor_height, total_dist, view_info).clamp(
                    clip.bottom,
                    clip.top
                );
                clip.bottom = lower_top;
            }
            if back_sector.ceiling_height > sector.ceiling_height {
                let upper_bottom = project_height(
                    sector.ceiling_height,
                    total_dist,
                    view_info
                ).clamp(clip.bottom, clip.top);
                clip.top = upper_bottom;
            }

            let angle = get_ray_angle(index, &player_cache.transform, view_info);
            let dir = Vec2::new(angle.cos(), angle.sin());
            let nudged_origin = hit.pos + dir * 0.05;
            let mut nudged_transform = player_cache.transform.clone();
            nudged_transform.translation = nudged_origin.extend(0.0);

            if
                let Some(next_hit) = get_single_hit(
                    &nudged_transform,
                    view_info,
                    back.sector,
                    map,
                    index
                )
            {
                let next_total_dist = total_dist + next_hit.perp_dist;
                render_wall_column(
                    index,
                    player_cache,
                    next_total_dist,
                    &next_hit,
                    clip,
                    view_info,
                    light,
                    map,
                    commands,
                    pool,
                    sprite_query
                );
            }
        }
    }
}

pub fn shading_brightness(normal: Vec3, dist: f32, view_info: &ViewInfo, light: &Light) -> f32 {
    let light_dir = light.direction.normalize();
    let ndotl = normal.normalize().dot(-light_dir).max(0.0);

    let ambient = 0.8;
    let diffuse = ndotl * light.intensity;
    let directional_brightness = (ambient + diffuse).min(1.0);

    let t = (dist / view_info.max_distance).clamp(0.0, 1.0);
    let dist_falloff = 1.0 - t * 0.7;

    directional_brightness * dist_falloff
}

pub fn shading_tint(normal: Vec3, dist: f32, view_info: &ViewInfo, light: &Light) -> Color {
    let brightness = shading_brightness(normal, dist, view_info, light);
    let light_srgba = light.color.to_srgba();
    Color::srgba(
        brightness * light_srgba.red,
        brightness * light_srgba.green,
        brightness * light_srgba.blue,
        1.0
    )
}

pub fn wall_normal(line_def: &LineDef) -> Vec3 {
    let dir = (line_def.end - line_def.start).normalize();
    Vec2::new(dir.y, -dir.x).extend(0.0)
}

fn draw_wall_column(
    commands: &mut Commands,
    pool: &mut SpritePool,
    sprite_query: &mut Query<(&mut Sprite, &mut Transform, &mut Visibility)>,
    index: usize,
    x: f32,
    wall_top: f32,
    wall_bottom: f32,
    u: f32,
    texture: Handle<Image>,
    tex_size: Vec2,
    dist: f32,
    normal: Vec3,
    view_info: &ViewInfo,
    light: &Light
) {
    if (wall_top - wall_bottom).abs() < 0.5 {
        return;
    }
    let tiled_u = u.fract();
    let tint = shading_tint(normal, dist, view_info, light);

    let entity = pool.next(commands);
    let height = (wall_top - wall_bottom).abs().max(1.0);
    let center_y = (wall_top + wall_bottom) / 2.0;

    let left = (tiled_u * tex_size.x).min(tex_size.x - 1.0);
    let right = (left + 1.0).min(tex_size.x);

    if let Ok((mut sprite, mut transform, mut vis)) = sprite_query.get_mut(entity) {
        sprite.image = texture;
        sprite.color = tint;
        sprite.custom_size = Some(Vec2::new(column_width(index, view_info), height));
        sprite.rect = Some(Rect::new(left, 0.0, right, tex_size.y));
        transform.translation = Vec3::new(x, center_y, 0.0);
        *vis = Visibility::Visible;
    }
}

fn draw_floor(
    commands: &mut Commands,
    pool: &mut SpritePool,
    sprite_query: &mut Query<(&mut Sprite, &mut Transform, &mut Visibility)>,
    index: usize,
    x: f32,
    wall_bottom: f32,
    clip_bottom: f32,
    texture: Handle<Image>,
    tex_size: Vec2,
    dist: f32,
    view_info: &ViewInfo,
    light: &Light
) {
    if wall_bottom <= clip_bottom {
        return;
    }
    draw_wall_column(
        commands,
        pool,
        sprite_query,
        index,
        x,
        wall_bottom,
        clip_bottom,
        0.0, // flat single-sample floor strip — see texturing fundamentals guide for per-row version
        texture,
        tex_size,
        dist,
        FLOOR_NORMAL,
        view_info,
        light
    );
}

fn draw_ceiling(
    commands: &mut Commands,
    pool: &mut SpritePool,
    sprite_query: &mut Query<(&mut Sprite, &mut Transform, &mut Visibility)>,
    index: usize,
    x: f32,
    wall_top: f32,
    clip_top: f32,
    texture: Handle<Image>,
    tex_size: Vec2,
    dist: f32,
    view_info: &ViewInfo,
    light: &Light
) {
    if wall_top >= clip_top {
        return;
    }
    draw_wall_column(
        commands,
        pool,
        sprite_query,
        index,
        x,
        clip_top,
        wall_top,
        0.0,
        texture,
        tex_size,
        dist,
        CEILING_NORMAL,
        view_info,
        light
    );
}

fn wall_u(world_pos: Vec2, line_def: &LineDef) -> f32 {
    (world_pos - line_def.start).length() / TILE_SIZE
}

pub fn hit_to_screen_x(view_info: &ViewInfo, ray_index: usize) -> f32 {
    let angle = get_ray_offset(ray_index, view_info);
    view_info.view_distance * angle.tan()
}

pub fn column_width(index: usize, view_info: &ViewInfo) -> f32 {
    let x_curr = hit_to_screen_x(view_info, index);
    // compare to the next column over to find the actual on-screen gap
    let next_index = (index + 1).min(RAY_COUNT - 1);
    let x_next = hit_to_screen_x(view_info, next_index);
    (x_next - x_curr).abs().max(1.0)
}
