use bevy_ecs::query::QueryData;
use bevy_state::prelude::in_state;
use valence::prelude::*;

use crate::{r#match::MatchState, GameState, Team};

use super::{break_blocks::BedDestroyedEvent, death::PlayerDeathEvent};

pub struct ChatPlugin;

impl Plugin for ChatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (handle_death_message, handle_bed_destroyed).run_if(in_state(GameState::Match)),
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
    mut events: EventReader<PlayerDeathEvent>,

    match_state: Res<MatchState>,
) {
    for event in events.read() {
        let Ok(victim) = chat_query.get(event.victim) else {
            continue;
        };
        let victim_eliminated = match_state
            .teams
            .get(&victim.team.name)
            .unwrap()
            .bed_destroyed;
        if let Some(attacker) = event.attacker {
            let Ok(attacker) = chat_query.get(attacker) else {
                continue;
            };

            let msg = if victim_eliminated {
                format!(
                    "{}{} §awas eliminated by {}{}",
                    victim.team.color.text_color(),
                    victim.username,
                    attacker.team.color.text_color(),
                    attacker.username
                )
            } else {
                format!(
                    "{}{} §awas killed by {}{}",
                    victim.team.color.text_color(),
                    victim.username,
                    attacker.team.color.text_color(),
                    attacker.username
                )
            };

            for mut client in &mut chat_query {
                client.client.send_chat_message(msg.clone());
            }
        } else {
            let msg = if victim_eliminated {
                format!(
                    "{}{} §awas eliminated",
                    victim.team.color.text_color(),
                    victim.username
                )
            } else {
                format!(
                    "{}{} §adied",
                    victim.team.color.text_color(),
                    victim.username
                )
            };

            for mut client in &mut chat_query {
                client.client.send_chat_message(msg.clone());
            }
        }
    }
}

fn handle_bed_destroyed(mut clients: Query<ChatQuery>, mut events: EventReader<BedDestroyedEvent>) {
    for event in events.read() {
        let team = &event.team;
        let msg = format!(
            "§aBed of team {}{} §awas destroyed!",
            team.color.text_color(),
            team.name
        );

        for mut client in &mut clients {
            client.client.send_chat_message(msg.clone());
        }
    }
}
