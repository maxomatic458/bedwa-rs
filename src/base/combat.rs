use bevy_ecs::query::QueryData;
use valence::{
    entity::{living::Health, EntityId, EntityStatuses},
    inventory::{player_inventory::PlayerInventory, HeldItem},
    prelude::*,
    protocol::{packets::play::EntityDamageS2c, sound::SoundCategory, Sound, WritePacket},
};

use crate::utils::item_kind::ItemStackExt;

use super::{death::IsDead, fall_damage::FallingState};

const ATTACK_COOLDOWN_TICKS: i64 = 0;

const KNOCKBACK_DEFAULT_XZ: f32 = 8.0;
const KNOCKBACK_DEFAULT_Y: f32 = 6.432;
const KNOCKBACK_SPEED_XY: f32 = 18.0;
const KNOCKBACK_SPEED_Y: f32 = 8.432;
const CRIT_MULTIPLIER: f32 = 1.5;

/// Attached to every client.
#[derive(Component, Default)]
pub struct CombatState {
    last_attacked_tick: i64,
    is_sprinting: bool,
}

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(EventLoopUpdate, combat_system);
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
struct CombatQuery {
    // client: &'static mut Client,
    entity_id: &'static EntityId,
    health: &'static mut Health,
    pos: &'static Position,
    state: &'static mut CombatState,
    statuses: &'static mut EntityStatuses,
    inventory: &'static Inventory,
    held_item: &'static HeldItem,
    falling_state: &'static FallingState,
    entity: Entity,
}

fn combat_system(
    commands: Commands,
    mut all_clients: Query<&mut Client>,
    mut clients: Query<CombatQuery, Without<IsDead>>,
    mut sprinting: EventReader<SprintEvent>,
    mut interact_entity_events: EventReader<InteractEntityEvent>,
    server: Res<Server>,
) {
    for &SprintEvent { client, state } in sprinting.read() {
        if let Ok(mut client) = clients.get_mut(client) {
            client.state.is_sprinting = state == SprintState::Start;
        }
    }

    for &InteractEntityEvent {
        client: attacker_ent,
        entity: victim_ent,
        interact,
        ..
    } in interact_entity_events.read()
    {
        if !matches!(interact, EntityInteraction::Attack) {
            continue;
        }
        let Ok([mut attacker, mut victim]) = clients.get_many_mut([attacker_ent, victim_ent])
        else {
            continue;
        };

        let dir = (victim.pos.0 - attacker.pos.0).normalize().as_vec3();

        let xz_knockback = if attacker.state.is_sprinting {
            KNOCKBACK_SPEED_XY
        } else {
            KNOCKBACK_DEFAULT_XZ
        };

        let y_knockback = if attacker.state.is_sprinting {
            KNOCKBACK_SPEED_Y
        } else {
            KNOCKBACK_DEFAULT_Y
        };

        let Ok(mut victim_client) = all_clients.get_mut(victim_ent) else {
            continue;
        };

        if attacker.state.last_attacked_tick + ATTACK_COOLDOWN_TICKS >= server.current_tick() {
            continue;
        }

        attacker.state.last_attacked_tick = server.current_tick();

        victim_client.set_velocity([dir.x * xz_knockback, y_knockback, dir.z * xz_knockback]);

        victim_client.trigger_status(EntityStatus::PlayAttackSound);
        victim.statuses.trigger(EntityStatus::PlayAttackSound);

        for mut client in &mut all_clients {
            client.play_sound(
                Sound::EntityPlayerHurt,
                SoundCategory::Hostile,
                victim.pos.0,
                1.0,
                1.0,
            );
        }

        let victim_id = victim.entity_id.get().into();
        let attacker_id = attacker.entity_id.get().into();
        let attacker_pos = attacker.pos.0.into();

        let attack_weapon = attacker.inventory.slot(attacker.held_item.slot());

        let weapon_damage = attack_weapon.damage();
        // let weapon_knockback = attack_weapon.knockback();

        victim.health.0 -= weapon_damage
            * if attacker.falling_state.falling {
                CRIT_MULTIPLIER
            } else {
                1.0
            };

        for mut client in all_clients.iter_mut() {
            // the red hit animation entity thing
            client.write_packet(&EntityDamageS2c {
                entity_id: victim_id,
                source_type_id: 1.into(), // idk what 1 is, probably physical damage
                source_cause_id: attacker_id,
                source_direct_id: attacker_id,
                source_pos: attacker_pos,
            });
        }
    }
}
