use valence::prelude::*;

pub struct Consumable {
    pub item: ItemStack,
    pub consumption_time_millis: u64,
    pub after_consumption: ItemStack,
}

#[derive(Debug, Event)]
pub struct ConsumptionEvent {
    pub consumed: ItemStack,
    pub consumer: Entity,
}

// pub struct ConsumptionTimer(pub Timer);

// pub struct ConsumablePlugin;

// impl Plugin for ConsumablePlugin {
//     fn build(&self, app: &mut App) {
//         app.add_systems(Update, start_consumption);
//     }
// }

// fn start_consumption(

// )
