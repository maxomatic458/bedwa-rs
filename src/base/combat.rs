use bevy_ecs::query::QueryData;
use valence::{
    entity::{living::Health, EntityId, EntityStatuses},
    inventory::HeldItem,
    prelude::*,
    protocol::{packets::play::EntityDamageS2c, WritePacket},
};

use crate::commands;

use super::on_death::IsDead;

const ATTACK_COOLDOWN_TICKS: i64 = 0;

const KNOCKBACK_DEFAULT_XZ: f32 = 8.0;
const KNOCKBACK_DEFAULT_Y: f32 = 6.432;
const KNOCKBACK_SPEED_XY: f32 = 18.0;
const KNOCKBACK_SPEED_Y: f32 = 8.432;

/// Attached to every client.
#[derive(Component, Default)]
pub struct CombatState;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(EventLoopUpdate, combat_system);
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
struct CombatQuery {
    client: &'static mut Client,
    entity_id: &'static EntityId,
    health: &'static mut Health,
    pos: &'static Position,
    state: &'static CombatState,
    statuses: &'static mut EntityStatuses,
    inventory: &'static Inventory,
    held_item: &'static HeldItem,
    entity: Entity,

}

fn combat_system(
    mut commands: Commands,
    // mut all_clients: Query<&mut Client>,
    mut clients: Query<CombatQuery>,
    mut sprinting: EventReader<SprintEvent>,
    mut interact_entity_events: EventReader<InteractEntityEvent>,
    
) {
    for &InteractEntityEvent {
        client: attacker,
        entity: victim,
        ..
    } in interact_entity_events.read()
    {
        let Ok([attacker, mut victim]) = clients.get_many_mut([attacker, victim]) else {
            continue;
        };

        let mut knockback_bonus = false;
        for &SprintEvent { state, .. } in sprinting.read() {
            knockback_bonus = state == SprintState::Start;
        }

        let dir = (victim.pos.0 - attacker.pos.0).normalize().as_vec3();

        let xz_knockback = if knockback_bonus {
            KNOCKBACK_SPEED_XY
        } else {
            KNOCKBACK_DEFAULT_XZ
        };

        let y_knockback = if knockback_bonus {
            KNOCKBACK_SPEED_Y
        } else {
            KNOCKBACK_DEFAULT_Y
        };

        victim
            .client
            .set_velocity([dir.x * xz_knockback, y_knockback, dir.z * xz_knockback]);

        victim.client.trigger_status(EntityStatus::PlayAttackSound);
        victim.statuses.trigger(EntityStatus::PlayAttackSound);

        // attacker.client.play_sound(
        //     Sound::EntityPlayerHurt,
        //     SoundCategory::Hostile,
        //     attacker.pos.0,
        //     1.0,
        //     1.0,
        // );

        // victim.client.play_sound(
        //     Sound::EntityPlayerHurt,
        //     SoundCategory::Hostile,
        //     victim.pos.0,
        //     1.0,
        //     1.0,
        // );

        let victim_id = victim.entity_id.get().into();
        let attacker_id = attacker.entity_id.get().into();
        let attacker_pos = attacker.pos.0.into();

        

        // let (_, attacker_pos, attacker_id, mut attacker_health, mut attacker_client, mut attacker_entity_status, mut attacker_anims) = attacker;
        // let (_, victim_pos, victim_id, mut victim_health, mut victim_client, mut victim_entity_status, mut victim_anims) = victim;

        // let attacker_pos = attacker_pos.0;
        // let victim_pos = victim_pos.0;

        // let dir = (victim_pos - attacker_pos).normalize();

        // victim_client.play_sound(
        //     Sound::EntityPlayerHurt,
        //     SoundCategory::Hostile,
        //     victim_pos,
        //     1.0,
        //     1.0,
        // );

        // client.play_hurt_anim

        // attacker_client.

        // attacker_client.write_packet(&Animation);

        // victim_health.0 -= 1.0;

        // attacker_client.write_packet(&valence::protocol::play:);

        // victim_anims.set(EntityAnimation::EnchantedHit, true);

        // victim_client.trigger_status(EntityStatus::);

        // attacker_client.trigger_status(EntityStatus::PlayAttackSound);
        // attacker_entity_status.trigger(EntityStatus::PlayAttackSound);

        // attacker_client.

        // let knockback = if has_bonus_knockback {
        //     SPRINT_KNOCKBACK_MULTIPLIER * KNOCKBACK
        // } else {
        //     KNOCKBACK
        // };

        // let knockback = dir * knockback;

        // victim_client.set_velocity(knockback.as_vec3());

        // attacker_client.write_packet(&EntityDamageS2c {
        //     entity_id: victim_id.get().into(),
        //     source_type_id: 1.into(),
        //     source_cause_id: attacker_id.get().into(),
        //     source_direct_id: attacker_id.get().into(),
        //     source_pos: attacker_pos.into(),
        // });
        victim.health.0 = (victim.health.0-1.0).max(0.0);
        if victim.health.0 <= 0.0 {
            commands.entity(victim.entity).insert(IsDead);

        }
        for mut player in clients.iter_mut() {
            // the red hit animation entity thing
            player.client.write_packet(&EntityDamageS2c {
                entity_id: victim_id,
                source_type_id: 1.into(), // idk what 1 is, probably physical damage
                source_cause_id: attacker_id,
                source_direct_id: attacker_id,
                source_pos: attacker_pos,
            });
        }
    }
}
