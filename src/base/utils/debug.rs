use std::time::Instant;

use valence::prelude::*;

#[derive(Resource)]
struct TickStart(Instant);

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, (tick,))
            .add_systems(First, record_tick_start_time)
            .insert_resource(TickStart(Instant::now()));
    }
}

fn record_tick_start_time(mut commands: Commands) {
    commands.insert_resource(TickStart(Instant::now()));
}

fn tick(server: Res<Server>, time: Res<TickStart>) {
    let tick = server.current_tick();

    if tick % (i64::from(server.tick_rate().get()) / 2) == 0 {
        let millis = time.0.elapsed().as_secs_f32() * 1000.0;
        tracing::debug!("Tick={tick}, MSPT={millis:.04}ms");
    }
}
