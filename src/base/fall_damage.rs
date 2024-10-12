use valence::{
    entity::{living::Health, EntityId},
    math::Aabb,
    prelude::*,
    protocol::{packets::play::EntityDamageS2c, sound::SoundCategory, Sound, WritePacket},
};

use crate::utils::ray_cast::aabb_full_block_intersections;

use super::death::IsDead;

pub struct FallDamagePlugin;

impl Plugin for FallDamagePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_fall_damage);
    }
}

#[derive(Component, Default)]
pub struct FallingState {
    fall_start_y: f64,
    pub falling: bool,
}

#[allow(clippy::type_complexity)]
fn handle_fall_damage(
    mut all_clients: Query<&mut Client>,
    mut clients: Query<
        (
            &EntityId,
            &mut FallingState,
            &Position,
            &mut Health,
            &Hitbox,
        ),
        (Changed<Position>, Without<IsDead>),
    >,
    layers: Query<&ChunkLayer, With<EntityLayer>>,
) {
    for (player_id, mut fall_damage_state, position, mut health, hitbox) in &mut clients {
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
                    let damage = (blocks_fallen - 3.0).max(0.0);
                    health.0 -= damage as f32;

                    for mut client in &mut all_clients {
                        client.play_sound(
                            Sound::EntityPlayerHurt,
                            SoundCategory::Hostile,
                            position.0,
                            1.0,
                            1.0,
                        );

                        client.write_packet(&EntityDamageS2c {
                            entity_id: player_id.get().into(),
                            source_type_id: 1.into(), // idk what 1 is, probably physical damage
                            source_cause_id: 0.into(),
                            source_direct_id: 0.into(),
                            source_pos: None,
                        });
                    }
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
