use bevy_ecs::system::Commands;
use valence::{
    inventory::{ClickSlotEvent, ClientInventoryState},
    prelude::*,
};
// use valence_spatial::bvh::Bvh;

pub struct ItemMenuPlugin;

impl Plugin for ItemMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (open_menu, select_menu_item))
            .observe(close_menu);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Event)]
pub struct MenuItemSelect {
    pub client: Entity,
    pub idx: u16,
}

#[derive(Debug, Clone, Component)]
pub struct ItemMenu {
    /// item menu
    pub menu: Inventory,
    /// Inventory Entity
    inventory_instance: Option<Entity>,
}

impl ItemMenu {
    pub fn new(mut menu: Inventory) -> Self {
        menu.readonly = true;
        Self {
            menu,
            inventory_instance: None,
        }
    }
}

fn open_menu(mut commands: Commands, mut clients: Query<(Entity, &mut ItemMenu), Added<ItemMenu>>) {
    for (player, mut item_menu) in clients.iter_mut() {
        let inventory = commands.spawn(item_menu.menu.clone()).id();
        item_menu.inventory_instance = Some(inventory);

        commands
            .entity(player)
            .insert(OpenInventory::new(inventory));
    }
}

fn close_menu(
    _trigger: Trigger<OnRemove, OpenInventory>,
    mut commands: Commands,
    clients: Query<Entity, With<ItemMenu>>,
) {
    for player in clients.iter() {
        tracing::info!("Closing menu");

        commands.entity(player).remove::<ItemMenu>();
    }
}

// #[derive(Debug, Clone, PartialEq, Eq, Event)]
// pub struct MultiItemMenuItemSelect {
//     pub client: Entity,
//     pub menu_name: String,
//     pub idx: u16,
// }

// #[derive(Debug, Clone, Component)]
// pub struct MultiItemMenu {
//     /// Item menus
//     pub menus: HashMap<String, Inventory>,
//     /// Sound that will be player if an item is selected
//     pub selection_sound: Option<Sound>, // TODO: per item?
//     /// Currently opened menu
//     pub open_menu: Option<String>,
// }

fn select_menu_item(
    commands: Commands,
    mut clients: Query<(
        Entity,
        &mut ClientInventoryState,
        Mut<CursorItem>,
        &ItemMenu,
        &mut Client,
        &OpenInventory,
    )>,
    mut events: EventReader<ClickSlotEvent>,
    mut event_writer: EventWriter<MenuItemSelect>,
) {
    for event in events.read() {
        let selected_slot = event.slot_id;
        let Ok((player, inventory_state, cursor_item, item_menu, client, open_inv)) =
            clients.get_mut(event.client)
        else {
            continue;
        };

        if selected_slot as u16 >= item_menu.menu.slot_count() {
            continue;
        }

        event_writer.send(MenuItemSelect {
            client: player,
            idx: selected_slot as u16,
        });
    }
}
