// use bevy::prelude::*;

// use crate::*;
// use ray::*;
// use map::*;

// #[derive(Resource)]
// pub struct Light {
//     pub direction: Vec3,
//     pub color: Color,
//     pub intensity: f32,
// }

// pub fn render(
//     mut gizmos: Gizmos,
//     mut hits: ResMut<Hits>,
//     view_info: Res<ViewInfo>,
//     player_cache: Res<PlayerCameraCache>,
//     map: Res<Map>,
//     light: Res<Light>,
//     mut clip: ResMut<Vclip>
// ) {
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
//             render_column(
//                 i,
//                 &player_cache,
//                 hit.perp_dist,
//                 hit,
//                 clip.0[i],
//                 &view_info,
//                 &mut gizmos,
//                 &light,
//                 &map
//             );
//         }
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

// pub fn render_column(
//     index: usize,
//     player_cache: &PlayerCameraCache,
//     total_dist: f32,
//     hit: &WallHit,
//     mut clip: VBounds,
//     view_info: &ViewInfo,
//     gizmos: &mut Gizmos,
//     light: &Light,
//     map: &Map
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

//             if let SectorType::ObstacleSector = hit.sector_type {
//                 // --- Draw the obstacle's front face ---
//                 let color = shade_color_directional(
//                     hit.line_def.front_side_def.middle_texture.unwrap(),
//                     wall_normal(&hit.line_def),
//                     total_dist,
//                     view_info,
//                     light
//                 );
//                 let room = &map.sectors[hit.room_sector_id]; // the containing room
//                 gizmos.line_2d(
//                     Vec2::new(x, wall_top_screen),
//                     Vec2::new(x, wall_bottom_screen),
//                     color
//                 );

//                 // --- Shrink clip around the obstacle's screen footprint ---
//                 let mut behind_clip = clip;
//                 behind_clip.bottom = wall_top_screen; // nothing behind can draw below the obstacle's top
//                 // (if the obstacle doesn't reach the floor, you'd also clip.top = wall_bottom_screen
//                 //  for a second segment below it — skipping that for now assuming floor-to-top boxes)

//                 // --- Continue the SAME ray, in the room sector the obstacle sits in, past it ---
//                 let angle = get_ray_angle(index, &player_cache.transform, view_info);
//                 let dir = Vec2::new(angle.cos(), angle.sin());
//                 let nudged_origin = hit.pos + dir * 0.05;
//                 let mut nudged_transform = player_cache.transform.clone();
//                 nudged_transform.translation = nudged_origin.extend(0.0);

//                 // Note: pass the room's sector_index the obstacle belongs to, not hit.sector_id
//                 // (hit.sector_id is the obstacle's own index — you need the containing room's index here)
//                 if
//                     let Some(next_hit) = get_single_hit(
//                         &nudged_transform,
//                         view_info,
//                         sector.id, // see note below — you need this on WallHit
//                         map,
//                         index
//                     )
//                 {
//                     let next_total_dist = total_dist + next_hit.perp_dist;
//                     render_column(
//                         index,
//                         player_cache,
//                         next_total_dist,
//                         &next_hit,
//                         behind_clip,
//                         view_info,
//                         gizmos,
//                         light,
//                         map
//                     );
//                 }

//                 draw_floor(
//                     x,
//                     wall_bottom_screen,
//                     clip.bottom,
//                     room.floor_texture,
//                     total_dist,
//                     view_info,
//                     light,
//                     gizmos
//                 );
//                 // no draw_ceiling here yet — see top-face section below
//                 return; // don't fall through to normal solid-wall floor/ceiling draw
//             }

//             let color = shade_color_directional(
//                 hit.line_def.front_side_def.middle_texture.unwrap(),
//                 wall_normal(&hit.line_def),
//                 total_dist,
//                 &view_info,
//                 light
//             );
//             // Solid wall —> draw within the clip window, then stop.
//             gizmos.line_2d(Vec2::new(x, wall_top_screen), Vec2::new(x, wall_bottom_screen), color);

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

// pub fn wall_normal(line_def: &LineDef) -> Vec3 {
//     let dir = (line_def.end - line_def.start).normalize();
//     Vec2::new(dir.y, -dir.x).extend(0.0)
// }

use bevy::prelude::*;

use crate::*;
use ray::*;
use map::*;

#[derive(Resource)]
pub struct Light {
    pub direction: Vec3,
    pub color: Color,
    pub intensity: f32,
}

#[derive(Resource, Default)]
pub struct SpritePool {
    pub entities: Vec<Entity>,
    pub used: usize,
}

impl SpritePool {
    pub fn next(&mut self, commands: &mut Commands) -> Entity {
        if self.used >= self.entities.len() {
            let e = commands
                .spawn((Sprite::default(), Transform::default(), Visibility::Hidden))
                .id();
            self.entities.push(e);
        }
        let e = self.entities[self.used];
        self.used += 1;
        e
    }

    pub fn reset(&mut self) {
        self.used = 0;
    }

    pub fn hide_unused(&self, query: &mut Query<(&mut Sprite, &mut Transform, &mut Visibility)>) {
        for &e in &self.entities[self.used..] {
            if let Ok((_, _, mut vis)) = query.get_mut(e) {
                *vis = Visibility::Hidden;
            }
        }
    }
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
            render_column(
                i,
                &player_cache,
                hit.perp_dist,
                hit,
                clip.0[i],
                &view_info,
                &mut commands,
                &mut pool,
                &mut sprite_query,
                &light,
                &map
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

/// Returns just the brightness multiplier (0.0-1.0ish), for tinting a sprite.
pub fn shading_brightness(normal: Vec3, dist: f32, view_info: &ViewInfo, light: &Light) -> f32 {
    let light_dir = light.direction.normalize();
    let ndotl = normal.normalize().dot(-light_dir).max(0.0);

    let ambient = 0.8;
    let diffuse = ndotl * light.intensity;
    let directional_brightness = (ambient + diffuse).min(1.0);

    let max_dist = view_info.max_distance;
    let t = (dist / max_dist).clamp(0.0, 1.0);
    let dist_falloff = 1.0 - t * 0.7;

    directional_brightness * dist_falloff
}

/// Builds a tint color (white * brightness * light color) to multiply against a sprite's texture.
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

/// Places/updates a pooled sprite as a single vertical textured column.
pub fn draw_textured_column(
    commands: &mut Commands,
    pool: &mut SpritePool,
    sprite_query: &mut Query<(&mut Sprite, &mut Transform, &mut Visibility)>,
    texture: Handle<Image>,
    x: f32,
    y_top: f32,
    y_bottom: f32,
    tex_u: f32, // 0.0-1.0, which horizontal strip of the texture to sample
    tex_u_width: f32,
    tint: Color,
    tex_size: Vec2
) {
    if (y_top - y_bottom).abs() < 0.5 {
        return;
    }
    let entity = pool.next(commands);
    let height = (y_top - y_bottom).abs().max(1.0);
    let center_y = (y_top + y_bottom) / 2.0;

    if let Ok((mut sprite, mut transform, mut vis)) = sprite_query.get_mut(entity) {
        sprite.image = texture;
        sprite.color = tint;
        sprite.custom_size = Some(Vec2::new(1.0, height));
        sprite.rect = Some(
            Rect::new(tex_u * tex_size.x, 0.0, (tex_u + tex_u_width) * tex_size.x, tex_size.y)
        );
        transform.translation = Vec3::new(x, center_y, 0.0);
        *vis = Visibility::Visible;
    }
}

fn draw_floor(
    commands: &mut Commands,
    pool: &mut SpritePool,
    sprite_query: &mut Query<(&mut Sprite, &mut Transform, &mut Visibility)>,
    x: f32,
    wall_bottom: f32,
    clip_bottom: f32,
    texture: Handle<Image>,
    dist: f32,
    view_info: &ViewInfo,
    light: &Light,
    index: usize,
    tex_size: Vec2
) {
    if wall_bottom > clip_bottom {
        let tint = shading_tint(FLOOR_NORMAL, dist, view_info, light);
        draw_textured_column(
            commands,
            pool,
            sprite_query,
            texture,
            x,
            wall_bottom,
            clip_bottom,
            (index as f32) / (RAY_COUNT as f32),
            1.0 / (RAY_COUNT as f32),
            tint,
            tex_size
        );
    }
}

fn draw_ceiling(
    commands: &mut Commands,
    pool: &mut SpritePool,
    sprite_query: &mut Query<(&mut Sprite, &mut Transform, &mut Visibility)>,
    x: f32,
    wall_top: f32,
    clip_top: f32,
    texture: Handle<Image>,
    dist: f32,
    view_info: &ViewInfo,
    light: &Light,
    index: usize,
    tex_size: Vec2
) {
    if wall_top < clip_top {
        let tint = shading_tint(CEILING_NORMAL, dist, view_info, light);
        draw_textured_column(
            commands,
            pool,
            sprite_query,
            texture,
            x,
            wall_top,
            clip_top,
            (index as f32) / (RAY_COUNT as f32),
            1.0 / (RAY_COUNT as f32),
            tint,
            tex_size
        );
    }
}

// Adjust to your actual texture pixel dimensions, or store per-Sector/SideDef if they vary.
const DEFAULT_TEX_SIZE: Vec2 = Vec2::new(4096.0, 4096.0);

pub fn render_column(
    index: usize,
    player_cache: &PlayerCameraCache,
    total_dist: f32,
    hit: &WallHit,
    mut clip: VBounds,
    view_info: &ViewInfo,
    commands: &mut Commands,
    pool: &mut SpritePool,
    sprite_query: &mut Query<(&mut Sprite, &mut Transform, &mut Visibility)>,
    light: &Light,
    map: &Map
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

            if let SectorType::ObstacleSector = hit.sector_type {
                let room = &map.sectors[hit.room_sector_id];

                let obstacle_top_screen = project_height(
                    room.floor_height + sector.ceiling_height,
                    total_dist,
                    view_info
                ).clamp(clip.bottom, clip.top);
                let obstacle_bottom_screen = project_height(
                    room.floor_height,
                    total_dist,
                    view_info
                ).clamp(clip.bottom, clip.top);

                let tint = shading_tint(wall_normal(&hit.line_def), total_dist, view_info, light);
                draw_textured_column(
                    commands,
                    pool,
                    sprite_query,
                    hit.line_def.front_side_def.middle_texture.clone().unwrap(),
                    x,
                    obstacle_top_screen,
                    obstacle_bottom_screen,
                    (index as f32) / (RAY_COUNT as f32),
                    1.0 / (RAY_COUNT as f32),
                    tint,
                    DEFAULT_TEX_SIZE
                );

                let mut behind_clip = clip;
                behind_clip.bottom = obstacle_top_screen;

                let angle = get_ray_angle(index, &player_cache.transform, view_info);
                let dir = Vec2::new(angle.cos(), angle.sin());
                let nudged_origin = hit.pos + dir * 0.05;
                let mut nudged_transform = player_cache.transform.clone();
                nudged_transform.translation = nudged_origin.extend(0.0);

                if
                    let Some(next_hit) = get_single_hit(
                        &nudged_transform,
                        view_info,
                        hit.room_sector_id,
                        map,
                        index
                    )
                {
                    let next_total_dist = total_dist + next_hit.perp_dist;
                    render_column(
                        index,
                        player_cache,
                        next_total_dist,
                        &next_hit,
                        behind_clip,
                        view_info,
                        commands,
                        pool,
                        sprite_query,
                        light,
                        map
                    );
                }

                draw_floor(
                    commands,
                    pool,
                    sprite_query,
                    x,
                    obstacle_bottom_screen,
                    clip.bottom,
                    room.floor_texture.clone(),
                    total_dist,
                    view_info,
                    light,
                    index,
                    DEFAULT_TEX_SIZE
                );
                return;
            }

            let tint = shading_tint(wall_normal(&hit.line_def), total_dist, view_info, light);
            draw_textured_column(
                commands,
                pool,
                sprite_query,
                hit.line_def.front_side_def.middle_texture.clone().unwrap(),
                x,
                wall_top_screen,
                wall_bottom_screen,
                (index as f32) / (RAY_COUNT as f32),
                1.0 / (RAY_COUNT as f32),
                tint,
                DEFAULT_TEX_SIZE
            );

            draw_floor(
                commands,
                pool,
                sprite_query,
                x,
                wall_bottom_screen,
                clip.bottom,
                sector.floor_texture.clone(),
                total_dist,
                view_info,
                light,
                index,
                DEFAULT_TEX_SIZE
            );
            draw_ceiling(
                commands,
                pool,
                sprite_query,
                x,
                wall_top_screen,
                clip.top,
                sector.ceiling_texture.clone(),
                total_dist,
                view_info,
                light,
                index,
                DEFAULT_TEX_SIZE
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
                x,
                wall_bottom_screen,
                clip.bottom,
                sector.floor_texture.clone(),
                total_dist,
                view_info,
                light,
                index,
                DEFAULT_TEX_SIZE
            );
            draw_ceiling(
                commands,
                pool,
                sprite_query,
                x,
                wall_top_screen,
                clip.top,
                sector.ceiling_texture.clone(),
                total_dist,
                view_info,
                light,
                index,
                DEFAULT_TEX_SIZE
            );

            if back_sector.ceiling_height < sector.ceiling_height {
                let upper_bottom = project_height(
                    back_sector.ceiling_height,
                    total_dist,
                    view_info
                ).clamp(clip.bottom, clip.top);
                if let Some(texture) = &hit.line_def.front_side_def.upper_texture {
                    let tint = shading_tint(
                        wall_normal(&hit.line_def),
                        total_dist,
                        view_info,
                        light
                    );
                    draw_textured_column(
                        commands,
                        pool,
                        sprite_query,
                        texture.clone(),
                        x,
                        wall_top_screen,
                        upper_bottom,
                        (index as f32) / (RAY_COUNT as f32),
                        1.0 / (RAY_COUNT as f32),
                        tint,
                        DEFAULT_TEX_SIZE
                    );
                }
                clip.top = upper_bottom;
            }

            if back_sector.floor_height > sector.floor_height {
                let lower_top = project_height(
                    back_sector.floor_height,
                    total_dist,
                    view_info
                ).clamp(clip.bottom, clip.top);
                if let Some(texture) = &hit.line_def.front_side_def.lower_texture {
                    let tint = shading_tint(
                        wall_normal(&hit.line_def),
                        total_dist,
                        view_info,
                        light
                    );
                    draw_textured_column(
                        commands,
                        pool,
                        sprite_query,
                        texture.clone(),
                        x,
                        lower_top,
                        wall_bottom_screen,
                        (index as f32) / (RAY_COUNT as f32),
                        1.0 / (RAY_COUNT as f32),
                        tint,
                        DEFAULT_TEX_SIZE
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
                    &map,
                    index
                )
            {
                let next_total_dist = total_dist + next_hit.perp_dist;
                render_column(
                    index,
                    player_cache,
                    next_total_dist,
                    &next_hit,
                    clip,
                    view_info,
                    commands,
                    pool,
                    sprite_query,
                    light,
                    map
                );
            }
        }
    }
}
