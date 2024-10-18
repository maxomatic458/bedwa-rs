// use bevy_time::Time;
// use valence::{
//     entity::{item::Stack, Velocity},
//     prelude::*,
// };

// use super::drop_items::DroppedItemsPickupTimer;

// const ITEM_GRAVITY_MPSS: f32 = 16.0;
// const ITEM_TERMINAL_VELOCITY_MPS: f32 = 40.0;

// // TODO: make this work!

// pub struct ItemEntityPlugin;

// impl Plugin for ItemEntityPlugin {
//     fn build(&self, app: &mut App) {
//         app.add_systems(Update, (tick,));
//     }
// }

// fn tick(
//     commands: Commands,
//     time: Res<Time>,
//     mut items: Query<
//         (
//             Entity,
//             &Hitbox,
//             &mut Position,
//             &mut Velocity,
//             &mut DroppedItemsPickupTimer,
//         ),
//         With<Stack>,
//     >,
//     mut layers: Query<&mut ChunkLayer, With<EntityLayer>>,
//     server: Res<Server>,
// ) {
//     let layer = layers.single_mut();
//     for (entity, hitbox, position, velocity, mut dropped_pickup_timer) in items.iter_mut() {
//         dropped_pickup_timer.0.tick(time.delta());

//         // handle_entity_movement(
//         //     &time,
//         //     &mut position,
//         //     &mut velocity,
//         //     hitbox,
//         //     &mut layer,
//         //     true,
//         //     ITEM_GRAVITY_MPSS,
//         //     ITEM_TERMINAL_VELOCITY_MPS,
//         // );
//     }
// }

// // fn handle_entity_movement(
// //     time: &Time,
// //     position: &mut Position,
// //     velocity: &mut Velocity,
// //     hitbox: &Hitbox,
// //     chunk_layer: &mut ChunkLayer,
// //     collide_with_blocks: bool,
// //     gravity: f32,
// //     terminal_velocity: f32,
// // ) {
// //     let delta_velocity = vec3(0.0, -gravity * time.delta_seconds(), 0.0);
// //     let mut new_velocity = velocity.0 + delta_velocity;

// //     let now = std::time::Instant::now();

// //     let cols = thick_world_raycast(
// //         chunk_layer,
// //         **hitbox,
// //         delta_velocity,
// //         delta_velocity.length() as f64,
// //         0.1,
// //     );
// //     // if !cols.is_empty() {
// //     //     tracing::info!("cols: {:?}", cols);

// //     // }
// //     // let (col_pos, col_block_face_normal) = cols.first().unwrap();
// //     for (col_pos, col_block_face_normal) in cols {
// //         tracing::info!("col_block_face_normal: {:?}", col_block_face_normal);
// //         match (
// //             col_block_face_normal.x,
// //             col_block_face_normal.y,
// //             col_block_face_normal.z,
// //         ) {
// //             (0, -1, 0) | (0, 1, 0) => new_velocity = Vec3::ZERO,
// //             (1, 0, 0) | (-1, 0, 0) => new_velocity.x = 0.0,
// //             (0, 0, 1) | (0, 0, -1) => new_velocity.z = 0.0,
// //             _ => {}
// //         }
// //     }

// //     // if let Some((_, col_block_face_normal)) = cols.first() {
// //     //     match (col_block_face_normal.x, col_block_face_normal.y, col_block_face_normal.z) {
// //     //         (0, -1, 0) | (0, 1, 0) => new_velocity= Vec3::ZERO,
// //     //         (1, 0, 0) | (-1, 0, 0) => new_velocity.x = 0.0,
// //     //         (0, 0, 1) | (0, 0, -1) => new_velocity.z = 0.0,
// //     //         _ => {}
// //     //     }
// //     // }

// //     let new_position = position.0 + new_velocity.as_dvec3() * time.delta_seconds() as f64;

// //     position.set(new_position);
// //     velocity.0 = new_velocity.clamp_length_max(terminal_velocity);
// // }
