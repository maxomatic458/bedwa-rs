use bevy_time::{Time, Timer, TimerMode};
use rand::Rng;
use valence::{
    entity::{ender_pearl::EnderPearlEntityBundle, entity::NoGravity, Velocity},
    interact_item::InteractItemEvent,
    inventory::HeldItem,
    prelude::*,
    protocol::{sound::SoundCategory, Sound},
};

use crate::{
    base::{
        armor::EquipmentExtReduction,
        bow::calculate_projectile_velocity,
        combat::{CombatState, EYE_HEIGHT, SNEAK_EYE_HEIGHT},
        death::PlayerHurtEvent,
        fall_damage::FallingState,
        physics::{
            CollidesWithBlocks, EntityBlockCollisionEvent, Gravity,
            PhysicsMarker, TerminalVelocity,
        },
    },
    utils::{despawn_timer::DespawnTimer, inventory::InventoryExt},
    Spectator,
};


const ENDER_PEARL_INACCURACY: f64 = 1.0;
const ENDER_PEARL_GRAVITY: f32 = 20.0;
const ENDER_PEARL_TERMINAL_VELOCITY: f32 = 100.0;
const ENDER_PEARL_BASE_DMG: f32 = 5.0;
const ENDER_PEARL_COOLDOWN_SECS: f32 = 1.0;

pub struct EnderPearlPlugin;

#[derive(Debug, Component)]
pub struct EnderPearlOwner(pub Entity);

#[derive(Debug, Component)]
struct EnderPearlTimer(pub Timer);

impl Default for EnderPearlTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(
            ENDER_PEARL_COOLDOWN_SECS,
            TimerMode::Once,
        ))
    }
}

impl Plugin for EnderPearlPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (throw_ender_pearl, on_hit_block, tick_pearl_timer));
    }
}

#[allow(clippy::type_complexity)]
fn throw_ender_pearl(
    mut commands: Commands,
    mut clients: Query<
        (
            Entity,
            &EntityLayerId,
            &mut Inventory,
            &HeldItem,
            &Position,
            &CombatState,
            &Look,
            Option<&mut EnderPearlTimer>,
        ),
        Without<Spectator>,
    >,
    mut events: EventReader<InteractItemEvent>,
    time: Res<Time>,
    mut layer: Query<&mut ChunkLayer>,
) {
    for event in events.read() {
        let Ok((
            player_ent,
            layer_id,
            mut inventory,
            held_item,
            position,
            combat_state,
            look,
            pearl_timer,
        )) = clients.get_mut(event.client)
        else {
            continue;
        };

        if let Some(mut timer) = pearl_timer {
            if !timer.0.tick(time.delta()).finished() {
                continue;
            }
        }

        let stack = inventory.slot(held_item.slot());

        if stack.item != ItemKind::EnderPearl {
            continue;
        }

        inventory.try_remove_all(&ItemStack::new(ItemKind::EnderPearl, 1, None));

        let mut layer = layer.single_mut();

        layer.play_sound(
            Sound::EntityEnderPearlThrow,
            SoundCategory::Neutral,
            position.0,
            0.5,
            rand::thread_rng().gen_range(0.333..0.5),
        );

        let yaw = look.yaw.to_radians();
        let pitch = look.pitch.to_radians();

        let direction = Vec3::new(
            -yaw.sin() * pitch.cos(),
            -pitch.sin(),
            yaw.cos() * pitch.cos(),
        );

        let velocity = calculate_projectile_velocity(direction, 2.0, ENDER_PEARL_INACCURACY);

        let mut position = position.0;
        position.y += if combat_state.is_sneaking {
            SNEAK_EYE_HEIGHT
        } else {
            EYE_HEIGHT
        } as f64
            - 0.1;

        commands
            .spawn(EnderPearlEntityBundle {
                position: Position(position),
                velocity: Velocity(velocity),
                layer: *layer_id,
                entity_no_gravity: NoGravity(true),

                ..Default::default()
            })
            .insert(PhysicsMarker)
            .insert(CollidesWithBlocks(None))
            .insert(DespawnTimer::from_secs(50.0))
            .insert(EnderPearlOwner(player_ent))
            .insert(TerminalVelocity(ENDER_PEARL_TERMINAL_VELOCITY))
            .insert(Gravity(ENDER_PEARL_GRAVITY));

        commands
            .entity(player_ent)
            .insert(EnderPearlTimer::default());
    }
}

fn on_hit_block(
    mut commands: Commands,
    mut ender_pearls: Query<(Entity, &EnderPearlOwner)>,
    mut thrower: Query<(&mut Position, &Equipment, &mut FallingState)>,
    mut events: EventReader<EntityBlockCollisionEvent>,
    mut damage_writer: EventWriter<PlayerHurtEvent>,
) {
    for event in events.read() {
        let Ok((pearl, owner)) = ender_pearls.get_mut(event.entity) else {
            continue;
        };

        let Ok((mut position, equipment, mut falling_state)) = thrower.get_mut(owner.0) else {
            continue;
        };

        commands.entity(pearl).insert(Despawned);

        *falling_state = FallingState {
            fall_start_y: event.collision_pos.y,
            falling: false,
        };

        position.set(event.collision_pos);

        let damage = equipment.received_damage(ENDER_PEARL_BASE_DMG);

        damage_writer.send(PlayerHurtEvent {
            attacker: None,
            victim: owner.0,
            damage,
            position: event.collision_pos,
        });
    }
}

fn tick_pearl_timer(mut throwsers: Query<&mut EnderPearlTimer>, time: Res<Time>) {
    for mut timer in throwsers.iter_mut() {
        timer.0.tick(time.delta());
    }
}
