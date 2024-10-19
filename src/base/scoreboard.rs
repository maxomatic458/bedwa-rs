use std::collections::HashMap;

use bevy_state::prelude::in_state;
use valence::prelude::*;
use valence::protocol::packets::play::scoreboard_objective_update_s2c::ObjectiveRenderType;
use valence::scoreboard::*;

use crate::r#match::MatchState;
use crate::GameState;
use crate::Team;

pub struct ScoreboardPlugin;

#[derive(Debug, Clone, Component)]
pub struct BedwarsScoreboad {
    pub entries: HashMap<Team, ScoreboardEntry>,
}

#[derive(Debug, Clone)]
pub struct ScoreboardEntry {
    pub players_left: u32,
    pub bed_destroyed: bool,
}

impl Plugin for ScoreboardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            update_match_scoreboard
                .run_if(in_state(GameState::Match))
                .run_if(resource_exists::<MatchState>),
        );
    }
}

fn update_match_scoreboard(
    commands: Commands,
    players: Query<(&Team, &EntityLayerId)>,
    mut scoreboard: Query<(&mut BedwarsScoreboad, &mut ObjectiveScores)>,
    match_state: Res<MatchState>,
) {
    if !match_state.is_changed() {
        return;
    }

    let Some((_, layer_id)) = players.iter().next() else {
        return;
    };

    let updated_entries = players
        .iter()
        .map(|(team, _)| {
            let team_state = match_state.teams.get(&team.name).unwrap();
            let entry = ScoreboardEntry {
                players_left: team_state.players_alive.len() as u32,
                bed_destroyed: team_state.bed_destroyed,
            };
            (team.clone(), entry)
        })
        .collect::<HashMap<Team, ScoreboardEntry>>();

    let Ok((mut scoreboard, mut scores)) = scoreboard.get_single_mut() else {
        create_scoreboard(commands, updated_entries, *layer_id);
        return;
    };

    let mut new_score: HashMap<String, i32> = HashMap::new();

    for (team, entry) in &updated_entries {
        let bed_destroyed_char = if !entry.bed_destroyed {
            "§a✔§r"
        } else {
            "§4✘§r"
        };
        let team_name = format!("{}{}{}", team.color.text_color(), team.name, "§r");
        new_score.insert(
            format!("{} {}", bed_destroyed_char, team_name),
            entry.players_left as i32,
        );
    }

    *scores = ObjectiveScores::with_map(new_score);
    scoreboard.entries = updated_entries;

    // for (mut bedwars_scoreboard, mut scores) in scoreboard.iter_mut() {
    //     let mut layer_id_out = None;
    //     let entries = players.iter().map(|(team, layer_id)| {
    //         let team_state = match_state.teams.get(&team.name).unwrap();
    //         let entry = ScoreboardEntry {
    //             players_left: team_state.players_alive.len() as u32,
    //             bed_destroyed: team_state.bed_destroyed,
    //         };
    //         (team.clone(), entry)
    //     }).collect::<HashMap<Team, ScoreboardEntry>>();

    //     let mut new_score: HashMap<String, i32> = HashMap::new();

    //     for (team, entry) in &entries {
    //         let bed_destroyed_char = if !entry.bed_destroyed { "§a✔§r" } else { "§4✘§r" };
    //         let team_name = format!("{}{}{}", team.color.text_color(), team.name, "§r");
    //         new_score.insert(format!("{} {}", bed_destroyed_char, team_name), entry.players_left as i32);
    //     }

    //     *scores = ObjectiveScores::with_map(new_score);

    //     bedwars_scoreboard.entries = entries;
    // }
}

fn create_scoreboard(
    mut commands: Commands,
    entries: HashMap<Team, ScoreboardEntry>,
    layer_id: EntityLayerId,
) -> Entity {
    let scores = ObjectiveScores::with_map(
        entries
            .iter()
            .map(|(team, entry)| {
                let bed_destroyed_char = if !entry.bed_destroyed {
                    "§a✔§r"
                } else {
                    "§4✘§r"
                };
                let team_name = format!("{}{}{}", team.color.text_color(), team.name, "§r");
                (
                    format!("{} {}", bed_destroyed_char, team_name),
                    entry.players_left as i32,
                )
            })
            .collect::<HashMap<String, i32>>(),
    );

    commands
        .spawn(ObjectiveBundle {
            name: Objective::new("bedwars".to_string()),
            display: ObjectiveDisplay("§a§lBedwars".into()),
            render_type: ObjectiveRenderType::Integer,
            scores,
            layer: layer_id,
            ..Default::default()
        })
        .insert(BedwarsScoreboad { entries })
        .id()
}
