use bevy_ecs::system::Commands;
use valence::{inventory::ClickSlotEvent, prelude::*};
// use valence_spatial::bvh::Bvh;

pub struct ItemMenuPlugin;

impl Plugin for ItemMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (open_menu, select_menu_item))
            .add_event::<MenuItemSelect>()
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
}

impl ItemMenu {
    pub fn new(mut menu: Inventory) -> Self {
        menu.readonly = true;
        Self { menu }
    }
}

fn open_menu(mut commands: Commands, mut clients: Query<(Entity, &mut ItemMenu), Added<ItemMenu>>) {
    for (player, item_menu) in clients.iter_mut() {
        let inventory = commands.spawn(item_menu.menu.clone()).id();

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

        if selected_slot as u16 >= item_menu.menu.slot_count() {
            continue;
        }

        event_writer.send(MenuItemSelect {
            client: player,
            idx: selected_slot as u16,
        });
    }
}
