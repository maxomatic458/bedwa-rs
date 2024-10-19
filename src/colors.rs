use std::str::FromStr;

use serde::{Deserialize, Serialize};
use valence::{
    command::parsers::{CommandArg, CommandArgParseError},
    nbt::{compound, Value},
    protocol::packets::play::command_tree_s2c::StringArg,
    ItemKind, ItemStack,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum TeamColor {
    White,
    Orange,
    Magenta,
    LightBlue,
    Yellow,
    Lime,
    Pink,
    Gray,
    LightGray,
    Cyan,
    Purple,
    Blue,
    Brown,
    Green,
    Red,
    Black,
}

impl std::fmt::Display for TeamColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let color = match self {
            TeamColor::White => "white",
            TeamColor::Orange => "orange",
            TeamColor::Magenta => "magenta",
            TeamColor::LightBlue => "light_blue",
            TeamColor::Yellow => "yellow",
            TeamColor::Lime => "lime",
            TeamColor::Pink => "pink",
            TeamColor::Gray => "gray",
            TeamColor::LightGray => "light_gray",
            TeamColor::Cyan => "cyan",
            TeamColor::Purple => "purple",
            TeamColor::Blue => "blue",
            TeamColor::Brown => "brown",
            TeamColor::Green => "green",
            TeamColor::Red => "red",
            TeamColor::Black => "black",
        };
        write!(f, "{}", color)
    }
}

impl FromStr for TeamColor {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "white" => Ok(TeamColor::White),
            "orange" => Ok(TeamColor::Orange),
            "magenta" => Ok(TeamColor::Magenta),
            "light_blue" => Ok(TeamColor::LightBlue),
            "yellow" => Ok(TeamColor::Yellow),
            "lime" => Ok(TeamColor::Lime),
            "pink" => Ok(TeamColor::Pink),
            "gray" => Ok(TeamColor::Gray),
            "light_gray" => Ok(TeamColor::LightGray),
            "cyan" => Ok(TeamColor::Cyan),
            "purple" => Ok(TeamColor::Purple),
            "blue" => Ok(TeamColor::Blue),
            "brown" => Ok(TeamColor::Brown),
            "green" => Ok(TeamColor::Green),
            "red" => Ok(TeamColor::Red),
            "black" => Ok(TeamColor::Black),
            _ => Err(()),
        }
    }
}

impl TeamColor {
    /// Convert the TeamColor to a hex value
    /// based on https://anrar4.github.io/DyeLeatherArmor/
    pub fn hex(&self) -> u32 {
        match self {
            TeamColor::White => 0xF9FFFE,
            TeamColor::Orange => 0xF9801D,
            TeamColor::Magenta => 0xC74EBD,
            TeamColor::LightBlue => 0x3AB3DA,
            TeamColor::Yellow => 0xFED83D,
            TeamColor::Lime => 0x80C71F,
            TeamColor::Pink => 0xF38BAA,
            TeamColor::Gray => 0x474F52,
            TeamColor::LightGray => 0x9D9D97,
            TeamColor::Cyan => 0x169C9C,
            TeamColor::Purple => 0x8932B8,
            TeamColor::Blue => 0x3C44AA,
            TeamColor::Brown => 0x835432,
            TeamColor::Green => 0x5E7C16,
            TeamColor::Red => 0xB02E26,
            TeamColor::Black => 0x1D1D21,
        }
    }

    pub fn wool_block(&self) -> ItemKind {
        match self {
            TeamColor::White => ItemKind::WhiteWool,
            TeamColor::Orange => ItemKind::OrangeWool,
            TeamColor::Magenta => ItemKind::MagentaWool,
            TeamColor::LightBlue => ItemKind::LightBlueWool,
            TeamColor::Yellow => ItemKind::YellowWool,
            TeamColor::Lime => ItemKind::LimeWool,
            TeamColor::Pink => ItemKind::PinkWool,
            TeamColor::Gray => ItemKind::GrayWool,
            TeamColor::LightGray => ItemKind::LightGrayWool,
            TeamColor::Cyan => ItemKind::CyanWool,
            TeamColor::Purple => ItemKind::PurpleWool,
            TeamColor::Blue => ItemKind::BlueWool,
            TeamColor::Brown => ItemKind::BrownWool,
            TeamColor::Green => ItemKind::GreenWool,
            TeamColor::Red => ItemKind::RedWool,
            TeamColor::Black => ItemKind::BlackWool,
        }
    }

    pub fn wool_carpet(&self) -> ItemKind {
        match self {
            TeamColor::White => ItemKind::WhiteCarpet,
            TeamColor::Orange => ItemKind::OrangeCarpet,
            TeamColor::Magenta => ItemKind::MagentaCarpet,
            TeamColor::LightBlue => ItemKind::LightBlueCarpet,
            TeamColor::Yellow => ItemKind::YellowCarpet,
            TeamColor::Lime => ItemKind::LimeCarpet,
            TeamColor::Pink => ItemKind::PinkCarpet,
            TeamColor::Gray => ItemKind::GrayCarpet,
            TeamColor::LightGray => ItemKind::LightGrayCarpet,
            TeamColor::Cyan => ItemKind::CyanCarpet,
            TeamColor::Purple => ItemKind::PurpleCarpet,
            TeamColor::Blue => ItemKind::BlueCarpet,
            TeamColor::Brown => ItemKind::BrownCarpet,
            TeamColor::Green => ItemKind::GreenCarpet,
            TeamColor::Red => ItemKind::RedCarpet,
            TeamColor::Black => ItemKind::BlackCarpet,
        }
    }

    pub fn bed_block(&self) -> ItemKind {
        match self {
            TeamColor::White => ItemKind::WhiteBed,
            TeamColor::Orange => ItemKind::OrangeBed,
            TeamColor::Magenta => ItemKind::MagentaBed,
            TeamColor::LightBlue => ItemKind::LightBlueBed,
            TeamColor::Yellow => ItemKind::YellowBed,
            TeamColor::Lime => ItemKind::LimeBed,
            TeamColor::Pink => ItemKind::PinkBed,
            TeamColor::Gray => ItemKind::GrayBed,
            TeamColor::LightGray => ItemKind::LightGrayBed,
            TeamColor::Cyan => ItemKind::CyanBed,
            TeamColor::Purple => ItemKind::PurpleBed,
            TeamColor::Blue => ItemKind::BlueBed,
            TeamColor::Brown => ItemKind::BrownBed,
            TeamColor::Green => ItemKind::GreenBed,
            TeamColor::Red => ItemKind::RedBed,
            TeamColor::Black => ItemKind::BlackBed,
        }
    }

    pub fn text_color(&self) -> &'static str {
        match self {
            TeamColor::White => "§f",
            TeamColor::Orange => "§6",
            TeamColor::Magenta => "§d",
            TeamColor::LightBlue => "§b",
            TeamColor::Yellow => "§e",
            TeamColor::Lime => "§a",
            TeamColor::Pink => "§d",
            TeamColor::Gray => "§8",
            TeamColor::LightGray => "§7",
            TeamColor::Cyan => "§3",
            TeamColor::Purple => "§5",
            TeamColor::Blue => "§9",
            TeamColor::Brown => "§6",
            TeamColor::Green => "§2",
            TeamColor::Red => "§c",
            TeamColor::Black => "§0",
        }
    }

    /// Converts neutral colorable items to the team color,
    /// such as leather armor and wool.
    pub fn to_team_item_stack(&self, stack: ItemStack) -> ItemStack {
        match stack.item {
            ItemKind::LeatherHelmet
            | ItemKind::LeatherChestplate
            | ItemKind::LeatherLeggings
            | ItemKind::LeatherBoots => {
                let mut nbt = stack.nbt.unwrap_or_default();
                nbt.merge(compound! {
                    "display" => compound! {
                        "color" => Value::Int(self.hex() as i32)
                    }
                });

                ItemStack::new(stack.item, stack.count, Some(nbt))
            }
            ItemKind::WhiteWool => ItemStack::new(self.wool_block(), stack.count, stack.nbt),
            _ => stack,
        }
    }
}

impl CommandArg for TeamColor {
    fn display() -> valence::protocol::packets::play::command_tree_s2c::Parser {
        valence::protocol::packets::play::command_tree_s2c::Parser::String(StringArg::SingleWord)
    }

    fn parse_arg(
        input: &mut valence::command::parsers::ParseInput,
    ) -> Result<Self, CommandArgParseError> {
        let color = input.clone().into_inner().to_lowercase();
        TeamColor::from_str(&color).map_err(|_| CommandArgParseError::InvalidArgument {
            expected: "a valid color".to_string(),
            got: "qdh".to_string(),
        })
    }
}
