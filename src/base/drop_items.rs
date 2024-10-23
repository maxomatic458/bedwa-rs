use std::collections::HashMap;

use bevy_ecs::{
    entity::Entity,
    event::EventReader,
    query::With,
    system::{Commands, Query, Res},
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
    ChunkLayer, Despawned, EntityLayer, ItemStack, Server,
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
        app.add_systems(Update, (drop_items, merge_to_stacks));
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

fn merge_to_stacks(
    mut commands: Commands,
    mut items: Query<(Entity, &Position, &mut Stack)>,
    server: Res<Server>,
) {
    if server.current_tick() % 4 != 0 {
        return;
    }

    // Contains tuples of two entities that will be merged together (into the first one)
    let mut to_merge = HashMap::new();

    for (entity1, position1, stack1) in items.iter() {
        for (entity2, position2, stack2) in items.iter() {
            if entity1 == entity2 {
                continue;
            }

            // Check if the two items are close enough to be merged
            if (position2.0 - position1.0).length() > 0.5 {
                continue;
            }

            // Check if the two items are the same type
            if stack1.0.item != stack2.0.item || stack1.0.nbt != stack2.0.nbt {
                continue;
            }

            let can_stack = stack1.count.saturating_add(stack2.count) <= stack1.item.max_stack();

            if can_stack {
                let key = entity1.min(entity2);
                let value = entity1.max(entity2);

                to_merge.insert(key, value);
            }
        }
    }

    for (add_items, remove_items) in to_merge {
        let Ok([mut entity1, entity2]) = items.get_many_mut([add_items, remove_items]) else {
            continue;
        };

        *entity1.2 = Stack(ItemStack::new(
            entity1.2.item,
            entity1.2.count.saturating_add(entity2.2.count),
            entity1.2.nbt.clone(),
        ));
        commands.entity(entity2.0).insert(Despawned);
    }
}
