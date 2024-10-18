use bevy_ecs::query::QueryData;
use bevy_state::prelude::in_state;
use valence::{
    entity::{EntityId, EntityStatuses},
    inventory::HeldItem,
    prelude::*,
};

use crate::{utils::item_stack::ItemStackExtCombat, GameState};

use super::{
    death::{IsDead, PlayerHurtEvent},
    fall_damage::FallingState,
};

const ATTACK_COOLDOWN_TICKS: i64 = 10;

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
        app.add_systems(
            EventLoopUpdate,
            combat_system.run_if(in_state(GameState::Match)),
        );
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
struct CombatQuery {
    client: &'static mut Client,
    entity_id: &'static EntityId,
    // health: &'static mut Health,
    pos: &'static Position,
    state: &'static mut CombatState,
    statuses: &'static mut EntityStatuses,
    inventory: &'static Inventory,
    held_item: &'static HeldItem,
    falling_state: &'static FallingState,
    entity: Entity,
}

fn combat_system(
    // mut layer: Query<&mut ChunkLayer>,
    mut clients: Query<CombatQuery, Without<IsDead>>,
    mut sprinting: EventReader<SprintEvent>,
    mut interact_entity_events: EventReader<InteractEntityEvent>,
    server: Res<Server>,
    mut event_writer: EventWriter<PlayerHurtEvent>,
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

        if attacker.state.last_attacked_tick + ATTACK_COOLDOWN_TICKS >= server.current_tick() {
            continue;
        }

        attacker.state.last_attacked_tick = server.current_tick();

        victim
            .client
            .set_velocity([dir.x * xz_knockback, y_knockback, dir.z * xz_knockback]);

        victim.client.trigger_status(EntityStatus::PlayAttackSound);
        victim.statuses.trigger(EntityStatus::PlayAttackSound);

        let attack_weapon = attacker.inventory.slot(attacker.held_item.slot());

        let weapon_damage = attack_weapon.damage();

        let final_damage = weapon_damage
            * if attacker.falling_state.falling {
                CRIT_MULTIPLIER
            } else {
                1.0
            };

        event_writer.send(PlayerHurtEvent {
            attacker: Some(attacker.entity),
            victim: victim.entity,
            damage: final_damage,
            position: victim.pos.0,
        });
    }
}
