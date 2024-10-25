use bevy_ecs::query::QueryData;
use bevy_state::prelude::in_state;
use bevy_time::{Time, Timer, TimerMode};
use rand::Rng;
use valence::{
    entity::{entity::Flags, living::StuckArrowCount, EntityId, EntityStatuses, Velocity},
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
    armor::EquipmentExtReduction,
    bow::{ArrowOwner, ArrowPower, BowUsed},
    death::{IsDead, PlayerHurtEvent},
    fall_damage::FallingState,
    physics::EntityEntityCollisionEvent,
};

pub const EYE_HEIGHT: f32 = 1.62;
pub const SNEAK_EYE_HEIGHT: f32 = 1.54;

// const ATTACK_COOLDOWN_TICKS: i64 = 10;
pub const ATTACK_COOLDOWN_MILLIS: u64 = 500;
const CRIT_MULTIPLIER: f32 = 1.5;

const FRIENLDY_FIRE: bool = false;

const DEFAULT_KNOCKBACK: f32 = 0.4;

const BURN_DAMAGE_PER_SECOND: f32 = 1.0;

#[derive(Component)]
pub struct Burning {
    pub timer: Timer, // second timer
    pub repeats_left: u32,
    pub attacker: Option<Entity>,
}

impl Burning {
    pub fn new(seconds: f32, attacker: Option<Entity>) -> Self {
        Self {
            timer: Timer::from_seconds(1.0, TimerMode::Repeating),
            repeats_left: (seconds as u32).saturating_sub(1),
            attacker,
        }
    }
}

/// Attached to every client.
#[derive(Component)]
pub struct CombatState {
    pub last_attack: std::time::Instant,
    pub is_sprinting: bool,
    pub is_sneaking: bool,
    /// The last time the player was hit by an entity.
    pub last_hit: std::time::Instant,
}

impl Default for CombatState {
    fn default() -> Self {
        Self {
            last_attack: std::time::Instant::now(),
            is_sprinting: false,
            is_sneaking: false,
            last_hit: std::time::Instant::now(),
        }
    }
}

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (combat_system, arrow_hits, apply_fire_damage, on_start_burn)
                .run_if(in_state(GameState::Match)),
        )
        .observe(on_remove_burn);
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
pub struct CombatQuery {
    pub client: &'static mut Client,
    pub entity_id: &'static EntityId,
    pub position: &'static Position,
    pub velocity: &'static mut Velocity,
    pub state: &'static mut CombatState,
    pub statuses: &'static mut EntityStatuses,
    pub inventory: &'static Inventory,
    pub held_item: &'static HeldItem,
    pub falling_state: &'static FallingState,
    pub equipment: &'static Equipment,
    pub team: &'static Team,
    pub entity: Entity,
    pub stuck_arrow_count: &'static mut StuckArrowCount,
}

fn xy_knockback(damage_pos: DVec3, victim_pos: DVec3) -> (f32, f32) {
    let mut x = (damage_pos.x - victim_pos.x) as f32;
    let mut z = (damage_pos.z - victim_pos.z) as f32;

    while x * x + z * z < 1.0e-4 {
        x = (rand::random::<f32>() - rand::random::<f32>()) * 0.01;
        z = (rand::random::<f32>() - rand::random::<f32>()) * 0.01;
    }

    (x, z)
}

fn receive_knockback(
    victim: &mut CombatQueryItem<'_>,
    mut strength: f32,
    x: f32,
    z: f32,
    extra_knockback: Vec3,
) {
    strength *= 1.0 - victim.equipment.knockback_resistance();
    if strength <= 0.0 {
        return;
    }

    let movement = victim.velocity.0;
    let knockback = Vec3::new(x, 0.0, z).normalize() * strength;
    let y_knockback = if !victim.falling_state.falling {
        0.4_f32.min(movement.y / 2.0 + strength)
    } else {
        movement.y
    };
    let knockback = Vec3::new(
        movement.x / 2.0 - knockback.x,
        y_knockback,
        movement.z / 2.0 - knockback.z,
    );

    victim
        .client
        .set_velocity((knockback * 20.0) + extra_knockback); // ticks per sec to mps
}

fn combat_system(
    mut commands: Commands,
    mut clients: Query<CombatQuery, Without<IsDead>>,
    mut sprinting: EventReader<SprintEvent>,
    mut sneaking: EventReader<SneakEvent>,
    mut interact_entity_events: EventReader<InteractEntityEvent>,
    mut event_writer: EventWriter<PlayerHurtEvent>,
) {
    for &SprintEvent { client, state } in sprinting.read() {
        if let Ok(mut client) = clients.get_mut(client) {
            client.state.is_sprinting = state == SprintState::Start;
        }
    }

    for &SneakEvent { client, state } in sneaking.read() {
        if let Ok(mut client) = clients.get_mut(client) {
            client.state.is_sneaking = state == SneakState::Start;
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

        if (attacker.state.last_attack.elapsed().as_millis() as u64) < ATTACK_COOLDOWN_MILLIS {
            continue;
        }

        attacker.state.last_attack = std::time::Instant::now();
        victim.state.last_hit = std::time::Instant::now();

        let dir = (victim.position.0 - attacker.position.0)
            .normalize()
            .as_vec3();

        let attack_weapon = attacker.inventory.slot(attacker.held_item.slot());

        let burn_time = attack_weapon.burn_time();

        if burn_time > 0.0 {
            commands
                .entity(victim.entity)
                .insert(Burning::new(burn_time, Some(attacker.entity)));
        }

        let extra_knockback = attack_weapon.knockback_extra() * dir;
        let (x, z) = xy_knockback(attacker.position.0, victim.position.0);
        receive_knockback(&mut victim, DEFAULT_KNOCKBACK, x, z, extra_knockback);

        victim.client.trigger_status(EntityStatus::PlayAttackSound);
        victim.statuses.trigger(EntityStatus::PlayAttackSound);

        let weapon_damage = attack_weapon.damage();
        let damage = weapon_damage
            * if attacker.falling_state.falling {
                CRIT_MULTIPLIER
            } else {
                1.0
            };

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
    arrows: Query<
        (&Velocity, &ArrowPower, &ArrowOwner, &BowUsed, &OldPosition),
        Without<CombatState>,
    >,
    mut clients: Query<CombatQuery>,
    mut event_writer: EventWriter<PlayerHurtEvent>,
    mut layer: Query<&mut ChunkLayer>,
) {
    for event in events.read() {
        let Ok((arrow_velocity, arrow_power, arrow_owner, bow_used, old_pos)) =
            arrows.get(event.entity1)
        else {
            continue;
        };

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

        let Ok(mut victim) = clients.get_mut(event.entity2) else {
            continue;
        };

        let mut layer = layer.single_mut();

        layer.play_sound(
            Sound::EntityArrowHit,
            SoundCategory::Neutral,
            victim.position.0,
            1.0,
            rand::thread_rng().gen_range(1.0909..1.3333),
        );

        let power_level = bow_used
            .0
            .enchantments()
            .get(&Enchantment::Power)
            .copied()
            .unwrap_or(0);

        let punch_level = bow_used
            .0
            .enchantments()
            .get(&Enchantment::Punch)
            .copied()
            .unwrap_or(0);

        let burn_time = bow_used.0.burn_time();

        tracing::info!("burn_time: {}", burn_time);

        if burn_time > 0.0 {
            commands
                .entity(victim.entity)
                .insert(Burning::new(burn_time, Some(arrow_owner.0)));
        }

        let damage = arrow_power.damage(**arrow_velocity, power_level);
        let damage_after_armor = victim.equipment.received_damage(damage);

        let extra_knockback = arrow_power.knockback_extra(**arrow_velocity, punch_level);
        let (x, z) = xy_knockback(old_pos.get(), victim.position.0);
        receive_knockback(&mut victim, DEFAULT_KNOCKBACK, x, z, extra_knockback);

        event_writer.send(PlayerHurtEvent {
            attacker: Some(arrow_owner.0),
            victim: victim.entity,
            damage: damage_after_armor,
            position: victim.position.0,
        });

        commands.entity(event.entity1).insert(Despawned);
    }
}

fn apply_fire_damage(
    mut commands: Commands,
    mut burning: Query<(Entity, &mut Flags, &Position, &mut Burning)>,
    mut event_writer: EventWriter<PlayerHurtEvent>,
    time: Res<Time>,
) {
    for (entity, mut flags, position, mut burn) in burning.iter_mut() {
        if burn.timer.tick(time.delta()).finished() {
            burn.repeats_left -= 1;

            if burn.repeats_left == 0 {
                commands.entity(entity).remove::<Burning>();
                flags.set_on_fire(false);
            }

            tracing::info!("damage");

            event_writer.send(PlayerHurtEvent {
                attacker: burn.attacker,
                victim: entity,
                damage: BURN_DAMAGE_PER_SECOND,
                position: position.0,
            });
        }
    }
}

fn on_start_burn(mut burning: Query<&mut Flags, Added<Burning>>) {
    for mut flags in &mut burning {
        flags.set_on_fire(true);
    }
}

fn on_remove_burn(trigger: Trigger<OnRemove, Burning>, mut entities: Query<&mut Flags>) {
    if let Ok(mut flags) = entities.get_mut(trigger.entity()) {
        flags.set_on_fire(false);
    }
}
