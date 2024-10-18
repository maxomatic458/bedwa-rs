use bevy_ecs::system::Commands;
use valence::{
    inventory::{ClickMode, ClickSlotEvent},
    prelude::*,
};
// use valence_spatial::bvh::Bvh;

pub struct ItemMenuPlugin;

impl Plugin for ItemMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (open_menu, select_menu_item))
            .add_event::<MenuItemSelect>()
            .observe(close_by_player)
            .observe(close_menu);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Event)]
pub struct MenuItemSelect {
    pub client: Entity,
    pub idx: u16,
    pub click_mode: ClickMode,
}

#[derive(Debug, Clone, Component)]
pub struct ItemMenu {
    /// item menu
    inventory: Inventory,
    /// Inventory entity,
    inventory_ent: Option<Entity>,
}

impl ItemMenu {
    pub fn new(mut inventory: Inventory) -> Self {
        inventory.readonly = true;
        Self {
            inventory,
            inventory_ent: None,
        }
    }

    pub fn inventory_ent(&self) -> Option<Entity> {
        self.inventory_ent
    }
}

fn open_menu(mut commands: Commands, mut clients: Query<(Entity, &mut ItemMenu), Added<ItemMenu>>) {
    for (player, mut item_menu) in clients.iter_mut() {
        let inventory = commands.spawn(item_menu.inventory.clone()).id();
        item_menu.inventory_ent = Some(inventory);
        commands
            .entity(player)
            .insert(OpenInventory::new(inventory));
    }
}

fn close_by_player(
    trigger: Trigger<OnRemove, OpenInventory>,
    mut commands: Commands,
    clients: Query<Entity, With<ItemMenu>>,
) {
    commands.entity(trigger.entity()).remove::<ItemMenu>();
    // for player in clients.iter() {
    //     tracing::info!("Closing menu by player");

    //     commands.entity(player).remove::<ItemMenu>();
    // }
}

fn close_menu(
    trigger: Trigger<OnRemove, ItemMenu>,
    mut commands: Commands,
    clients: Query<(Entity, &Username), With<OpenInventory>>,
) {
    tracing::info!("#### closing menu");
    commands.entity(trigger.entity()).remove::<OpenInventory>();
    commands.entity(trigger.entity()).remove::<ItemMenu>();
}

fn select_menu_item(
    mut clients: Query<(Entity, &ItemMenu)>,
    mut events: EventReader<ClickSlotEvent>,
    mut event_writer: EventWriter<MenuItemSelect>,
) {
    for event in events.read() {
        let selected_slot = event.slot_id;
        let Ok((player, item_menu)) = clients.get_mut(event.client) else {
            continue;
        };

        if selected_slot as u16 >= item_menu.inventory.slot_count() {
            continue;
        }

        event_writer.send(MenuItemSelect {
            client: player,
            idx: selected_slot as u16,
            click_mode: event.mode,
        });
    }
}
