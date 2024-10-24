use bevy_ecs::query::QueryData;
use bevy_time::{Time, Timer, TimerMode};
use valence::{entity::Velocity, math::Aabb, prelude::*};

use crate::utils::{aabb::AabbExt, direction::DirectionExt, ray_cast::collide};

use super::drop_items::ITEM_GRAVITY_MPSS;

pub struct PhysicsPlugin;

/// Marker for entities that have physics
#[derive(Component)]
pub struct PhysicsMarker;

/// Component that can be attached to a entity with the `PhysicsMarker` to stop physics after the timer has elapsed
#[derive(Component)]
pub struct SimPhysicsForTime(pub Timer);

impl SimPhysicsForTime {
    pub fn for_secs(secs: f32) -> Self {
        Self(Timer::from_seconds(secs, TimerMode::Once))
    }
}

/// Marker for terminal velocity of an entity in m/s
#[derive(Component)]
pub struct TerminalVelocity(pub f32);

/// Marker for entities that have gravity
#[derive(Component)]
pub struct Gravity(pub f32);

impl Gravity {
    pub fn items() -> Self {
        Self(ITEM_GRAVITY_MPSS)
    }
}

/// Drag component that will multiply the entities velocity by the drag value every second
#[derive(Component)]
pub struct Drag(pub f32);

/// Marker for entities that can collide with blocks
/// The inner `Option<Aabb>` is a possible override for the hitbox used for collision detection
#[derive(Component)]
pub struct CollidesWithBlocks(pub Option<Aabb>);

/// Marker for entities that can collide with other entities
/// The inner `Option<Aabb>` is a possible override for the hitbox used for collision detection
#[derive(Component)]
pub struct CollidesWithEntities(pub Option<Aabb>);

/// Marker for entities that can be collided with
/// TODO: The inner `Option<Aabb>` is a possible override for the hitbox used for collision detection
#[derive(Component)]
pub struct CollidableForEntities;

/// Component that will disable physics if the item collides with a block
/// on one of the sides specified in the `sides` field
#[derive(Component)]
pub struct GetsStuckOnCollision {
    pub sides: u8,
}

impl GetsStuckOnCollision {
    pub fn all() -> Self {
        Self { sides: u8::MAX }
    }

    pub fn none() -> Self {
        Self { sides: 0 }
    }

    pub fn ground() -> Self {
        Self { sides: 1 << 1 }
    }
}

/// Event that will be emitted when an entity collides with another entity
#[derive(Debug, Event)]
pub struct EntityEntityCollisionEvent {
    pub entity1: Entity,
    pub entity2: Entity,
}

/// Event that will be emitted when an entity collides with a block
#[derive(Debug, Event)]
pub struct EntityBlockCollisionEvent {
    pub entity: Entity,
    pub collision_pos: DVec3,
    pub previous_velocity: Vec3,
    pub block_pos: BlockPos,
    // The direction of the face the entity collided with
    pub direction: Direction,
}

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (handle_collision,))
            .add_event::<EntityBlockCollisionEvent>()
            .add_event::<EntityEntityCollisionEvent>();
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
struct PhysicsQuery {
    pub entity: Entity,
    pub old_position: &'static mut OldPosition,
    pub position: &'static mut Position,
    pub velocity: &'static mut Velocity,
    pub drag: Option<&'static Drag>,
    pub hitbox: &'static Hitbox,
    pub gravity: Option<&'static mut Gravity>,
    pub entity_collider: Option<&'static mut CollidesWithEntities>,
    pub block_collider: Option<&'static mut CollidesWithBlocks>,
    pub stuck_sides: Option<&'static GetsStuckOnCollision>,
    pub sim_physics_for_time: Option<&'static mut SimPhysicsForTime>,
    pub terminal_velocity: Option<&'static TerminalVelocity>,
}

#[allow(clippy::type_complexity)]
fn handle_collision(
    mut commands: Commands,
    time: Res<Time>,
    mut entity_block_collision_writer: EventWriter<EntityBlockCollisionEvent>,
    mut entity_collision_writer: EventWriter<EntityEntityCollisionEvent>,
    mut entities: Query<PhysicsQuery, With<PhysicsMarker>>,
    hittable_entities: Query<(Entity, &Hitbox), With<CollidableForEntities>>,
    layer: Query<&ChunkLayer, With<EntityLayer>>,
) {
    enum Event {
        MarkAsRemovePhysics(Entity),
        EntityBlockCollision(EntityBlockCollisionEvent),
        EntityEntityCollision(EntityEntityCollisionEvent),
    }

    let (tx, rx) = std::sync::mpsc::channel::<Event>();

    entities.par_iter_mut().for_each(|mut query| {
        if let Some(sim_physics_for_time) = query.sim_physics_for_time.as_mut() {
            sim_physics_for_time.0.tick(time.delta());
            if sim_physics_for_time.0.finished() {
                tx.send(Event::MarkAsRemovePhysics(query.entity)).unwrap();
                return;
            }
        }

        if let Some(drag) = query.drag {
            query.velocity.0 *= 1.0 - drag.0 * time.delta_seconds();
        }

        if let Some(ref gravity) = query.gravity {
            query.velocity.0.y -= gravity.0 * time.delta_seconds();
        }

        if let Some(terminal_velocity) = query.terminal_velocity {
            query.velocity.0 = query.velocity.0.clamp_length(0.0, terminal_velocity.0);
        }

        let layer = layer.single();

        let old_velocity = query.velocity.0;
        if let Some(mut block_collider) = query.block_collider {
            if let Some(ref mut override_hitbox) = block_collider.0 {
                *override_hitbox = override_hitbox.translate_to(query.position.0);
            }

            for _ in 0..3 {
                let velocity_delta = query.velocity.0 * time.delta_seconds();
                let (vx, vy, vz) = (velocity_delta.x, velocity_delta.y, velocity_delta.z);

                let step_x = if vx > 0.0 { 1 } else { -1 };
                let step_y = if vy > 0.0 { 1 } else { -1 };
                let step_z = if vz > 0.0 { 1 } else { -1 };

                let steps_x = (query.hitbox.get().width_x() / 2.0) as i32;
                let steps_y = (query.hitbox.get().height() / 2.0) as i32;
                let steps_z = (query.hitbox.get().width_z() / 2.0) as i32;

                let (x, y, z) = (
                    query.position.0.x as i32,
                    query.position.0.y as i32,
                    query.position.0.z as i32,
                );
                let (cx, cy, cz) = (
                    (query.position.0.x + velocity_delta.x as f64) as i32,
                    (query.position.0.y + velocity_delta.y as f64) as i32,
                    (query.position.0.z + velocity_delta.z as f64) as i32,
                );

                let mut potential_collisions = vec![];

                let x_range = if step_x > 0 {
                    x - step_x * (steps_x + 1)..=cx + step_x * (steps_x + 2)
                } else {
                    cx + step_x * (steps_x + 2)..=x - step_x * (steps_x + 1)
                };

                for i in x_range.step_by(step_x.unsigned_abs() as usize) {
                    let y_range = if step_y > 0 {
                        y - step_y * (steps_y + 1)..=cy + step_y * (steps_y + 2)
                    } else {
                        cy + step_y * (steps_y + 2)..=y - step_y * (steps_y + 1)
                    };

                    for j in y_range.step_by(step_y.unsigned_abs() as usize) {
                        let z_range = if step_z > 0 {
                            z - step_z * (steps_z + 1)..=cz + step_z * (steps_z + 2)
                        } else {
                            cz + step_z * (steps_z + 2)..=z - step_z * (steps_z + 1)
                        };

                        for k in z_range.step_by(step_z.unsigned_abs() as usize) {
                            let block_pos = BlockPos { x: i, y: j, z: k };

                            // if k % 32 == 0 {
                            // tracing::info!("block_pos: {:?}", block_pos);

                            // }

                            let block = layer.block(block_pos);

                            if let Some(block) = block {
                                if block.state.is_air() {
                                    continue;
                                }

                                for collider in block.state.collision_shapes() {
                                    let aabb = collider
                                        .translate(DVec3::new(i as f64, j as f64, k as f64));

                                    let (entry_time, normals) = collide(
                                        &block_collider.0.unwrap_or(query.hitbox.get()),
                                        velocity_delta,
                                        &aabb,
                                    );

                                    if normals.0.is_none()
                                        && normals.1.is_none()
                                        && normals.2.is_none()
                                    {
                                        continue;
                                    }

                                    potential_collisions.push((entry_time, normals, block_pos));
                                }
                            }
                        }
                    }
                }

                if potential_collisions.is_empty() {
                    break;
                }

                let (mut entry_time, normals, block_pos) = potential_collisions
                    .iter()
                    .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
                    .unwrap();
                entry_time -= 0.001;

                if let Some(normal_x) = normals.0 {
                    query.velocity.0.x = 0.0;
                    query.position.0.x += vx as f64 * entry_time;

                    if let Some(stuck_sides) = query.stuck_sides {
                        let dir_idx = if normal_x == 1 {
                            Direction::East.as_u8()
                        } else {
                            Direction::West.as_u8()
                        };

                        if stuck_sides.sides & (1 << dir_idx) != 0 {
                            query.velocity.0 = Vec3::ZERO;

                            if let Some(mut gravity) = query.gravity {
                                gravity.0 = 0.0;
                            }

                            tx.send(Event::EntityBlockCollision(EntityBlockCollisionEvent {
                                entity: query.entity,
                                collision_pos: query.position.0,
                                previous_velocity: old_velocity,
                                block_pos: *block_pos,
                                direction: Direction::from_u8(dir_idx),
                            }))
                            .unwrap();

                            break;
                        }
                    }
                }

                if let Some(normal_y) = normals.1 {
                    query.velocity.0.y = 0.0;
                    query.position.0.y += vy as f64 * entry_time;

                    if let Some(stuck_sides) = query.stuck_sides {
                        let dir_idx = if normal_y == 1 {
                            Direction::Up.as_u8()
                        } else {
                            Direction::Down.as_u8()
                        };

                        if stuck_sides.sides & (1 << dir_idx) != 0 {
                            query.velocity.0 = Vec3::ZERO;

                            if let Some(ref mut gravity) = query.gravity {
                                gravity.0 = 0.0;
                            }

                            tx.send(Event::EntityBlockCollision(EntityBlockCollisionEvent {
                                entity: query.entity,
                                collision_pos: query.position.0,
                                previous_velocity: old_velocity,
                                block_pos: *block_pos,
                                direction: Direction::from_u8(dir_idx),
                            }))
                            .unwrap();

                            break;
                        }
                    }
                }

                if let Some(normal_z) = normals.2 {
                    query.velocity.0.z = 0.0;
                    query.position.0.z += vz as f64 * entry_time;

                    if let Some(stuck_sides) = query.stuck_sides {
                        let dir_idx = if normal_z == 1 {
                            Direction::South.as_u8()
                        } else {
                            Direction::North.as_u8()
                        };

                        if stuck_sides.sides & (1 << dir_idx) != 0 {
                            query.velocity.0 = Vec3::ZERO;

                            if let Some(ref mut gravity) = query.gravity {
                                gravity.0 = 0.0;
                            }

                            tx.send(Event::EntityBlockCollision(EntityBlockCollisionEvent {
                                entity: query.entity,
                                collision_pos: query.position.0,
                                previous_velocity: old_velocity,
                                block_pos: *block_pos,
                                direction: Direction::from_u8(dir_idx),
                            }))
                            .unwrap();

                            break;
                        }
                    }
                }
            }

            if let Some(ref mut entity_collider) = query.entity_collider {
                if let Some(ref mut entity_collider_hitbox) = entity_collider.0 {
                    *entity_collider_hitbox = entity_collider_hitbox.translate_to(query.position.0);
                }
            }

            if let Some(ref query_entity_hitbox) = query.entity_collider {
                // tracing::info!("here");
                for (entity, hitbox) in hittable_entities.iter() {
                    if entity == query.entity {
                        continue;
                    }

                    let normals = collide(
                        &query_entity_hitbox.0.unwrap_or(query.hitbox.get()),
                        query.velocity.0 * time.delta_seconds(),
                        &hitbox.get(),
                    )
                    .1;

                    if normals.0.is_none() && normals.1.is_none() && normals.2.is_none() {
                        continue;
                    }

                    tx.send(Event::EntityEntityCollision(EntityEntityCollisionEvent {
                        entity1: query.entity,
                        entity2: entity,
                    }))
                    .unwrap();
                }
            }
        }

        query.position.0 += (query.velocity.0 * time.delta_seconds()).as_dvec3();
    });

    for event in rx.try_iter() {
        match event {
            Event::MarkAsRemovePhysics(entity) => {
                commands.entity(entity).remove::<PhysicsMarker>();
            }
            Event::EntityBlockCollision(event) => {
                entity_block_collision_writer.send(event);
            }
            Event::EntityEntityCollision(event) => {
                entity_collision_writer.send(event);
            }
        }
    }
}
