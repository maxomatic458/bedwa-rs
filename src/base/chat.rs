use bevy_ecs::query::QueryData;
use bevy_state::prelude::in_state;
use valence::prelude::*;

use crate::{
    r#match::{EndMatch, POST_MATCH_TIME_SECS},
    GameState, Team,
};

use super::{
    break_blocks::BedDestroyedEvent,
    death::{PlayerDeathEvent, PlayerEliminatedEvent},
};

pub struct ChatPlugin;

impl Plugin for ChatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                (handle_death_message, handle_bed_destroyed).run_if(in_state(GameState::Match)),
                handle_match_end,
            ),
        );
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
struct ChatQuery {
    client: &'static mut Client,
    username: &'static Username,
    team: &'static Team,
}

fn handle_death_message(
    mut chat_query: Query<ChatQuery>,
    mut death_events: EventReader<PlayerDeathEvent>,
    mut eliminated_events: EventReader<PlayerEliminatedEvent>,
) {
    for event in death_events.read() {
        let Ok(victim) = chat_query.get(event.victim) else {
            continue;
        };

        let msg = if let Some(attacker) = event.attacker {
            let Ok(attacker) = chat_query.get(attacker) else {
                continue;
            };

            format!(
                "{}§n{}§r §awas killed by {}{}",
                victim.team.color.text_color(),
                victim.username,
                attacker.team.color.text_color(),
                attacker.username
            )
        } else {
            format!(
                "{}§n{}§r §adied",
                victim.team.color.text_color(),
                victim.username
            )
        };

        for mut client in &mut chat_query {
            client.client.send_chat_message(&msg);
        }
    }

    for event in eliminated_events.read() {
        let Ok(victim) = chat_query.get(event.victim) else {
            continue;
        };

        let msg = if let Some(attacker) = event.attacker {
            let Ok(attacker) = chat_query.get(attacker) else {
                continue;
            };

            format!(
                "{}§n{}§r §awas eliminated by {}{}",
                victim.team.color.text_color(),
                victim.username,
                attacker.team.color.text_color(),
                attacker.username
            )
        } else {
            format!(
                "{}§n{}§r §aeliminated",
                victim.team.color.text_color(),
                victim.username
            )
        };

        for mut client in &mut chat_query {
            client.client.send_chat_message(&msg);
        }
    }
}

fn handle_bed_destroyed(mut clients: Query<ChatQuery>, mut events: EventReader<BedDestroyedEvent>) {
    for event in events.read() {
        let team = &event.team;
        let msg = format!(
            "§aBed of team {}§n{}§r §awas destroyed!",
            team.color.text_color(),
            team.name
        );

        for mut client in &mut clients {
            client.client.send_chat_message(&msg);
        }
    }
}

fn handle_match_end(mut clients: Query<ChatQuery>, mut events: EventReader<EndMatch>) {
    for event in events.read() {
        let team = &event.winner;

        let msg = format!(
            "§bTeam {}§n{}§r §bwon the match!",
            team.color.text_color(),
            team.name
        );

        let return_to_lobby_msg = format!(
            "§eReturning to lobby in {} seconds...",
            POST_MATCH_TIME_SECS.round() as i32
        );

        for mut client in &mut clients {
            tracing::debug!("Sending chat message to {}", client.username.0);
            client.client.send_chat_message(&msg);
            client.client.send_chat_message(&return_to_lobby_msg);
        }
    }
}
