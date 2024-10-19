use valence::{
    interact_item::InteractItemEvent,
    inventory::{player_inventory::PlayerInventory, HeldItem},
    prelude::*,
    protocol::sound::SoundCategory,
};

use super::armor::ItemKindExtArmor;

pub struct ArmorRightClickEquipPlugin;

impl Plugin for ArmorRightClickEquipPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, equip_armor_on_right_click);
    }
}

fn equip_armor_on_right_click(
    mut query: Query<(&mut Client, &Position, &mut Inventory, &HeldItem)>,
    mut events: EventReader<InteractItemEvent>,
) {
    for event in events.read() {
        let Ok((mut client, position, mut inventory, held_item)) = query.get_mut(event.client)
        else {
            continue;
        };

        // TODO: handle readonly inventory

        let stack = inventory.slot(held_item.slot()).clone();
        if !stack.item.is_armor() {
            continue;
        }

        let target_slot = if stack.item.is_helmet() {
            PlayerInventory::SLOT_HEAD
        } else if stack.item.is_chestplate() {
            PlayerInventory::SLOT_CHEST
        } else if stack.item.is_leggings() {
            PlayerInventory::SLOT_LEGS
        } else if stack.item.is_boots() {
            PlayerInventory::SLOT_FEET
        } else {
            continue;
        };

        // in vanilla this also plays when equipping it normally
        if let Some(equip_sound) = stack.item.equip_sound() {
            client.play_sound(equip_sound, SoundCategory::Player, position.0, 1.0, 1.0);
        }

        inventory.swap_slot(held_item.slot(), target_slot);
    }
}
