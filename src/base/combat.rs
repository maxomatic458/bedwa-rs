use bevy_ecs::query::QueryData;
use bevy_state::prelude::in_state;
use valence::{
    entity::{living::StuckArrowCount, EntityId, EntityStatuses, Velocity},
    inventory::HeldItem,
    prelude::*,
    protocol::{sound::SoundCategory, Sound},
};

use crate::{
    base::enchantments::{Enchantment, ItemStackExtEnchantments},
    utils::item_stack::ItemStackExtWeapons,
    GameState, Team,
};

use super::{
    armor::EquipmentExtDamageReduction,
    bow::{ArrowOwner, ArrowPower, BowUsed},
    death::{IsDead, PlayerHurtEvent},
    fall_damage::FallingState,
    physics::EntityEntityCollisionEvent,
};

const ATTACK_COOLDOWN_TICKS: i64 = 10;

const KNOCKBACK_DEFAULT_XZ: f32 = 8.0;
const KNOCKBACK_DEFAULT_Y: f32 = 6.432;
const KNOCKBACK_SPEED_XY: f32 = 18.0;
const KNOCKBACK_SPEED_Y: f32 = 8.432;
const CRIT_MULTIPLIER: f32 = 1.5;

const FRIENLDY_FIRE: bool = false;

/// Attached to every client.
#[derive(Component, Default)]
pub struct CombatState {
    pub last_attacked_tick: i64,
    pub is_sprinting: bool,
    /// The last tick the player was hit
    pub last_hit_tick: i64,
}

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (combat_system, arrow_hits).run_if(in_state(GameState::Match)),
        );
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
struct CombatQuery {
    client: &'static mut Client,
    entity_id: &'static EntityId,
    position: &'static Position,
    state: &'static mut CombatState,
    statuses: &'static mut EntityStatuses,
    inventory: &'static Inventory,
    held_item: &'static HeldItem,
    falling_state: &'static FallingState,
    equipment: &'static Equipment,
    team: &'static Team,
    entity: Entity,
    stuck_arrow_count: &'static mut StuckArrowCount,
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

        if attacker.team == victim.team && !FRIENLDY_FIRE {
            continue;
        }

        if attacker.state.last_attacked_tick + ATTACK_COOLDOWN_TICKS >= server.current_tick() {
            continue;
        }

        attacker.state.last_attacked_tick = server.current_tick();
        victim.state.last_hit_tick = server.current_tick();

        let dir = (victim.position.0 - attacker.position.0)
            .normalize()
            .as_vec3();

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

        let mut knockback_vec = Vec3::new(dir.x * xz_knockback, y_knockback, dir.z * xz_knockback);
        let attack_weapon = attacker.inventory.slot(attacker.held_item.slot());

        knockback_vec += attack_weapon.knockback_extra() * dir;

        victim.client.set_velocity(knockback_vec);

        victim.client.trigger_status(EntityStatus::PlayAttackSound);
        victim.statuses.trigger(EntityStatus::PlayAttackSound);

        let weapon_damage = attack_weapon.damage();
        let damage = weapon_damage
            * if attacker.falling_state.falling {
                CRIT_MULTIPLIER
            } else {
                1.0
            };

        tracing::info!("Dealing {} damage to {}", damage, victim.entity);

        let damage_after_armor = victim.equipment.received_damage(damage);

        event_writer.send(PlayerHurtEvent {
            attacker: Some(attacker.entity),
            victim: victim.entity,
            damage: damage_after_armor,
            position: victim.position.0,
        });
    }
}

fn arrow_hits(
    mut commands: Commands,
    mut events: EventReader<EntityEntityCollisionEvent>,
    arrows: Query<(&Velocity, &ArrowPower, &ArrowOwner, &BowUsed)>,
    mut clients: Query<CombatQuery>,
    mut event_writer: EventWriter<PlayerHurtEvent>,
) {
    for event in events.read() {
        let Ok((arrow_velocity, arrow_power, arrow_owner, bow_used)) = arrows.get(event.entity1)
        else {
            continue;
        };

        tracing::info!("Owner: {:?}", arrow_owner);

        tracing::info!("event: {:?}", event);

        let Ok(mut attacker) = clients.get_mut(arrow_owner.0) else {
            continue;
        };

        attacker.client.play_sound(
            Sound::EntityArrowHitPlayer,
            SoundCategory::Player,
            attacker.position.0,
            1.0,
            1.0,
        );

        let Ok(victim) = clients.get_mut(event.entity2) else {
            continue;
        };

        let power_level = bow_used
            .0
            .enchantments()
            .get(&Enchantment::Power)
            .copied()
            .unwrap_or(0);

        let damage = arrow_power.damage(**arrow_velocity, power_level);
        let damage_after_armor = victim.equipment.received_damage(damage);

        event_writer.send(PlayerHurtEvent {
            attacker: Some(arrow_owner.0),
            victim: victim.entity,
            damage: damage_after_armor,
            position: victim.position.0,
        });

        commands.entity(event.entity1).insert(Despawned);
    }
}
