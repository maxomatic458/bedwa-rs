use rand::Rng;
use valence::{
    entity::{
        arrow::ArrowEntityBundle, entity::NoGravity, living::LivingFlags, OnGround, Velocity,
    },
    event_loop::PacketEvent,
    interact_item::InteractItemEvent,
    inventory::{HeldItem, PlayerAction},
    math::Aabb,
    prelude::*,
    protocol::{packets::play::PlayerActionC2s, sound::SoundCategory, Sound},
};

use crate::{
    base::{
        combat::{EYE_HEIGHT, SNEAK_EYE_HEIGHT},
        enchantments::{Enchantment, ItemStackExtEnchantments},
        physics::TerminalVelocity,
    },
    utils::{despawn_timer::DespawnTimer, inventory::InventoryExt},
};

use super::{
    combat::CombatState,
    enchantments::power_extra_dmg,
    physics::{
        CollidesWithBlocks, CollidesWithEntities, Drag, EntityBlockCollisionEvent,
        GetsStuckOnCollision, Gravity, PhysicsMarker,
    },
};

pub struct BowPlugin;

const CAN_SHOOT_AFTER_MS: u64 = 150;
const ARROW_GRAVITY_MPSS: f32 = 20.0;
const BOW_INACCURACY: f64 = 0.0172275;
pub const ARROW_BASE_DAMAGE: f32 = 2.0;
const ARROW_TERMINAL_VELOCITY: f32 = 100.0;

/// The owner of the arrow
#[derive(Debug, Component)]
pub struct ArrowOwner(pub Entity);

/// The bow the arrow was shot from
#[derive(Debug, Component)]
pub struct BowUsed(pub ItemStack);

#[derive(Component)]
pub struct ArrowPower(pub f64);

impl ArrowPower {
    pub fn is_critical(&self) -> bool {
        self.0 >= 3.0
    }

    pub fn damage(&self, velocity_mps: Vec3, power_level: u32) -> f32 {
        let base = ARROW_BASE_DAMAGE + power_extra_dmg(power_level);

        let velocity_tps_len = velocity_mps.length() / 20.0; // m/s to m/tick

        let mut damage = (velocity_tps_len * base).clamp(0.0, i32::MAX as f32).ceil() as i32;

        tracing::info!("Arrow damage: {}", damage);

        if self.is_critical() {
            let crit_extra_damage: i32 = rand::thread_rng().gen_range(0..damage / 2 + 1);
            damage += crit_extra_damage;
        }

        damage as f32
    }

    pub fn knockback_extra(&self, mut velocity_mps: Vec3, punch_level: u32) -> Vec3 {
        velocity_mps.y = 0.0;
        velocity_mps.normalize() * punch_level as f32 * 0.6
    }
}

#[derive(Component)]
struct BowState {
    pub start_draw_tick: std::time::Instant,
}

impl BowState {
    pub fn new(start_draw: std::time::Instant) -> Self {
        Self {
            start_draw_tick: start_draw,
        }
    }
}

impl Plugin for BowPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                on_bow_draw,
                on_bow_release,
                on_shoot,
                on_hit_block,
                on_arrow_fly,
            ),
        )
        .add_event::<BowShootEvent>();
    }
}

fn on_bow_draw(
    mut commands: Commands,
    mut clients: Query<(&Inventory, &HeldItem, &mut LivingFlags)>,
    mut item_use_events: EventReader<InteractItemEvent>,
    // server: Res<Server>,
) {
    for event in item_use_events.read() {
        let Ok((inventory, held_item, mut flags)) = clients.get_mut(event.client) else {
            continue;
        };
        flags.set_using_item(false);

        let stack = inventory.slot(held_item.slot()).clone();
        if !inventory.check_contains_stack(&ItemStack::new(ItemKind::Arrow, 1, None), true)
            || stack.item != ItemKind::Bow
        {
            continue;
        }

        flags.set_using_item(true);

        let now = std::time::Instant::now();
        commands.entity(event.client).insert(BowState::new(now));
    }
}

#[derive(Debug, Event)]
pub struct BowShootEvent {
    pub client: Entity,
    pub position: Position,
    pub look: Look,
    pub ms_drawn: u64,
    pub bow_used: ItemStack,
}

fn on_bow_release(
    clients: Query<(&BowState, &Position, &Look, &Inventory, &HeldItem)>,
    mut packet_events: EventReader<PacketEvent>,
    mut event_writer: EventWriter<BowShootEvent>,
) {
    for packet in packet_events.read() {
        let Some(player_action) = packet.decode::<PlayerActionC2s>() else {
            continue;
        };

        let Ok((bow_state, position, look, inventory, held_item)) = clients.get(packet.client)
        else {
            continue;
        };

        let selected = inventory.slot(held_item.slot()).clone();
        if player_action.action != PlayerAction::ReleaseUseItem || selected.item != ItemKind::Bow {
            continue;
        }

        let ms_drawn = bow_state.start_draw_tick.elapsed().as_millis() as u64;

        tracing::info!("Bow drawn for {}ms", ms_drawn);

        let stack = inventory.slot(held_item.slot()).clone();

        event_writer.send(BowShootEvent {
            client: packet.client,
            position: *position,
            look: *look,
            ms_drawn,
            bow_used: stack,
        });
    }
}

fn on_shoot(
    mut shooter: Query<(&mut Inventory, &EntityLayerId, &CombatState)>,
    mut commands: Commands,
    mut shoot_events: EventReader<BowShootEvent>,
    mut layer: Query<&mut ChunkLayer>,
) {
    for event in shoot_events.read() {
        if event.ms_drawn < CAN_SHOOT_AFTER_MS {
            continue;
        }

        let mut layer = layer.single_mut();
        let sound_pitch = rand::thread_rng().gen_range(0.75..=1.125);

        layer.play_sound(
            Sound::EntityArrowShoot,
            SoundCategory::Neutral,
            event.position.0,
            sound_pitch,
            1.0,
        );

        let yaw = event.look.yaw.to_radians();
        let pitch = event.look.pitch.to_radians();

        let direction = Vec3::new(
            -yaw.sin() * pitch.cos(),
            -pitch.sin(),
            yaw.cos() * pitch.cos(),
        );

        let Ok((mut shooter_inv, layer_id, combat_state)) = shooter.get_mut(event.client) else {
            continue;
        };

        tracing::info!("{:?} shoots an arrow", event.client);

        if !event
            .bow_used
            .enchantments()
            .contains_key(&Enchantment::Infinity)
        {
            shooter_inv.try_remove_all(&ItemStack::new(ItemKind::Arrow, 1, None));
        }

        let arrow_power = get_bow_power_for_draw_ticks(event.ms_drawn / 50);
        let velocity = calculate_bow_velocity(direction, arrow_power as f32);

        let direction = velocity.normalize();

        let mut position = event.position.0;

        position.y += if combat_state.is_sneaking {
            SNEAK_EYE_HEIGHT
        } else {
            EYE_HEIGHT
        } as f64
            - 0.1;
        position += direction.as_dvec3() * 0.1;

        // Separate hitbox for arrow-block and arrow-entity collision
        let block_hitbox = Aabb::new(
            position - DVec3::new(0.001, 0.001, 0.001),
            position + DVec3::new(0.001, 0.001, 0.001),
        );

        commands
            .spawn(ArrowEntityBundle {
                position: Position(position),
                velocity: Velocity(velocity),

                layer: *layer_id,
                entity_no_gravity: NoGravity(true),
                ..Default::default()
            })
            .insert(PhysicsMarker)
            .insert(GetsStuckOnCollision::all())
            .insert(CollidesWithBlocks(Some(block_hitbox)))
            .insert(CollidesWithEntities(None))
            .insert(DespawnTimer::from_secs(50.0))
            .insert(ArrowOwner(event.client))
            .insert(BowUsed(event.bow_used.clone()))
            .insert(Drag(0.99 / 20.0))
            .insert(ArrowPower(arrow_power))
            .insert(TerminalVelocity(ARROW_TERMINAL_VELOCITY))
            .insert(Gravity(ARROW_GRAVITY_MPSS));
    }
}

#[allow(clippy::type_complexity)]
fn on_arrow_fly(
    query: Query<(&Position, &OnGround, &ArrowPower), (With<ArrowOwner>, With<ArrowPower>)>,
    mut layer: Query<&mut ChunkLayer>,
) {
    for (position, on_ground, power) in query.iter() {
        if on_ground.0 {
            continue;
        }

        let mut layer = layer.single_mut();

        if power.is_critical() {
            layer.play_particle(&Particle::Crit, true, position.0, Vec3::ZERO, 0.1, 1);
        }
    }
}

fn on_hit_block(
    mut arrows: Query<&mut OnGround, (With<ArrowOwner>, With<ArrowPower>)>,
    mut events: EventReader<EntityBlockCollisionEvent>,
    mut layer: Query<&mut ChunkLayer>,
) {
    for event in events.read() {
        let Ok(mut on_ground) = arrows.get_mut(event.entity) else {
            continue;
        };

        if **on_ground {
            continue;
        }

        on_ground.0 = true;

        let mut layer = layer.single_mut();

        layer.play_sound(
            Sound::EntityArrowHit,
            SoundCategory::Neutral,
            event.collision_pos,
            1.0,
            rand::thread_rng().gen_range(1.0909..1.3333),
        );
    }
}

fn calculate_bow_velocity(direction: Vec3, arrow_power: f32) -> Vec3 {
    let direction = direction.normalize();

    // We multiply by 20, because our velocity is m/s,
    // minecraft uses m/tick

    (direction
        + Vec3 {
            x: random_triangle(0.0, BOW_INACCURACY) as f32,
            y: random_triangle(0.0, BOW_INACCURACY) as f32,
            z: random_triangle(0.0, BOW_INACCURACY) as f32,
        })
        * arrow_power
        * 20.0
}

fn random_triangle(a: f64, b: f64) -> f64 {
    a + b * (rand::thread_rng().gen::<f64>() - rand::thread_rng().gen::<f64>())
}

fn get_bow_power_for_draw_ticks(ticks: u64) -> f64 {
    let power = ticks as f64 / 20.0;
    let power = (power * power + power * 2.0) / 3.0;

    power.clamp(0.1, 1.0) * 3.0
}
