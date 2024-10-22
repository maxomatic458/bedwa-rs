use bevy_ecs::{
    entity::Entity,
    event::EventReader,
    query::With,
    system::{Commands, Query},
};
use valence::{
    app::{Plugin, Update},
    entity::{
        entity::NoGravity,
        item::{ItemEntityBundle, Stack},
        player::PlayerEntity,
        EntityLayerId, Look, Position, Velocity,
    },
    inventory::DropItemStackEvent,
    math::{vec3, DVec3},
    ChunkLayer, EntityLayer,
};

use crate::utils::despawn_timer::DespawnTimer;

use super::{
    item_pickup::PickupMarker,
    physics::{CollidesWithBlocks, GetsStuckOnCollision, Gravity, PhysicsMarker},
};

const DROP_STRENGTH: f32 = 4.5;
const DROP_OFFSET: f64 = 1.6;

pub const ITEM_GRAVITY_MPSS: f32 = 16.0;

/// A marker for items that can be picked up once the inner time elapses.

pub struct ItemDropPlugin;

impl Plugin for ItemDropPlugin {
    fn build(&self, app: &mut valence::app::App) {
        app.add_systems(Update, (drop_items,));
    }
}

fn drop_items(
    mut commands: Commands,
    players: Query<(&Position, &Look), With<PlayerEntity>>,
    mut event_reader: EventReader<DropItemStackEvent>,
    layers: Query<Entity, (With<ChunkLayer>, With<EntityLayer>)>,
) {
    let layer = layers.single();
    for event in event_reader.read() {
        let Ok(player) = players.get(event.client) else {
            continue;
        };

        let (player_pos, player_look) = player;

        let item_dropped_from_pos = Position(player_pos.0 + DVec3::new(0.0, DROP_OFFSET, 0.0));

        let yaw = player_look.yaw.to_radians();
        let pitch = player_look.pitch.to_radians();

        let item_velocity = vec3(
            -yaw.sin() * pitch.cos(),
            -pitch.sin(),
            yaw.cos() * pitch.cos(),
        ) * DROP_STRENGTH;

        commands
            .spawn(ItemEntityBundle {
                item_stack: Stack(event.stack.clone()),
                position: Position(*item_dropped_from_pos),
                velocity: Velocity(item_velocity),

                layer: EntityLayerId(layer),
                entity_no_gravity: NoGravity(true),
                ..Default::default()
            })
            .insert(PickupMarker::default())
            .insert(Gravity::items())
            .insert(PhysicsMarker)
            .insert(CollidesWithBlocks(None))
            .insert(GetsStuckOnCollision::ground())
            .insert(DespawnTimer::items());
    }
}
