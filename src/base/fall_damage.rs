use bevy_state::prelude::in_state;
use valence::{client::UpdateClientsSet, math::Aabb, prelude::*};

use crate::{utils::ray_cast::aabb_full_block_intersections, GameState};

use super::death::{IsDead, PlayerHurtEvent};

pub struct FallDamagePlugin;

impl Plugin for FallDamagePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            handle_fall_damage
                .before(UpdateClientsSet)
                .run_if(in_state(GameState::Match)),
        );
    }
}

#[derive(Component, Default)]
pub struct FallingState {
    fall_start_y: f64,
    pub falling: bool,
}

#[allow(clippy::type_complexity)]
fn handle_fall_damage(
    mut clients: Query<
        (Entity, &mut FallingState, &Position, &Hitbox),
        (Changed<Position>, Without<IsDead>),
    >,
    layers: Query<&ChunkLayer, With<EntityLayer>>,
    mut event_writer: EventWriter<PlayerHurtEvent>,
) {
    for (player_ent, mut fall_damage_state, position, hitbox) in &mut clients {
        let layer = layers.single();

        let flattened_player_hitbox = Aabb::new(
            DVec3::new(hitbox.min().x, hitbox.min().y - 0.05, hitbox.min().z),
            DVec3::new(hitbox.max().x, hitbox.min().y, hitbox.max().z),
        );

        let blocks_below = aabb_full_block_intersections(&flattened_player_hitbox);

        if blocks_below.iter().any(|b| {
            if let Some(block) = layer.block(*b) {
                !block.state.is_air()
            } else {
                false
            }
        }) {
            // player is on ground
            if fall_damage_state.falling {
                let blocks_fallen = fall_damage_state.fall_start_y - position.0.y;
                if blocks_fallen >= 3.0 {
                    let damage = (blocks_fallen - 3.0).max(0.0) as f32;

                    event_writer.send(PlayerHurtEvent {
                        attacker: None,
                        victim: player_ent,
                        position: position.0,
                        damage,
                    });
                }

                fall_damage_state.fall_start_y = position.0.y;
                fall_damage_state.falling = false;
            }
        } else {
            // player is falling
            if fall_damage_state.fall_start_y <= position.0.y {
                fall_damage_state.fall_start_y = position.0.y
            } else {
                fall_damage_state.falling = true;
            }
        }
    }
}
