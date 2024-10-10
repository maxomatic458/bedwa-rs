use valence::{command, command_macros};

use command::parsers::CommandArg;
use command_macros::Command;

// TODO: maybe remove this

#[derive(Command, Debug, Clone)]
#[paths("bw", "bedwars")]
pub enum BedwarsCommand {
    /// Join a bedwars team
    #[paths = "team join {team_name}"]
    TeamJoin { team_name: String },
}
