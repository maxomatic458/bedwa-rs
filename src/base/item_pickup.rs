use bevy_ecs::{
    entity::Entity,
    system::{Commands, Query},
};
use valence::{
    app::{App, Plugin, Update},
    client::Client,
    entity::{item::Stack, Position},
    prelude::Inventory,
    protocol::{sound::SoundCategory, Sound},
    Despawned,
};

use crate::utils::inventory::InventoryExt;

use super::drop_items::DroppedItemsPickupTimer;

// https://minecraft.fandom.com/wiki/Item_(entity)

const PICKUP_RANGE_HOR: f64 = 1.0;
const PICKUP_RANGE_VER: f64 = 0.5;

pub struct ItemPickupPlugin;

impl Plugin for ItemPickupPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (pickup_items,));
    }
}

fn pickup_items(
    mut commands: Commands,
    mut players: Query<(&Position, &mut Inventory, &mut Client)>,
    mut items: Query<(Entity, &Position, &mut Stack, &DroppedItemsPickupTimer)>,
) {
    // This will be really inefficient, but for a bedwars server it probably won't matter
    for (player_pos, mut player_inv, mut client) in players.iter_mut() {
        for (item_entity, item_pos, mut stack, pickup_timer) in items.iter_mut() {
            let player_vec3 = player_pos.0;
            let item_vec3 = item_pos.0;

            let dx = player_vec3.x - item_vec3.x;
            let dy = player_vec3.y - item_vec3.y;
            let dz = player_vec3.z - item_vec3.z;

            if dx.abs() > PICKUP_RANGE_HOR
                || dy.abs() > PICKUP_RANGE_VER
                || dz.abs() > PICKUP_RANGE_HOR
            {
                continue;
            }

            if !pickup_timer.0.finished() {
                continue;
            }

            let picked_up = player_inv.try_pickup_stack(&stack.0);

            if picked_up == 0 {
                continue;
            }

            if picked_up as i8 == stack.0.count {
                commands.entity(item_entity).insert(Despawned);
            } else {
                stack.0.count -= picked_up as i8;
            }

            client.play_sound(
                Sound::EntityItemPickup,
                SoundCategory::Player,
                item_vec3,
                0.25,
                rand::random::<f32>() * 1.6 + 0.6,
            );
        }
    }
}
