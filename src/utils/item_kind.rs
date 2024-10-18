use valence::ItemKind;

use crate::colors::TeamColor;

pub trait ItemKindExt {
    /// Convert colorable blocks to the given color
    /// (e.g wool, beds)
    /// Will return the same item if it's not colorable
    fn to_colored(self, color: TeamColor) -> ItemKind;
    /// Is the block a bed
    fn is_bed(&self) -> bool;
    /// Is the block a wool block
    fn is_wool(&self) -> bool;
    /// Is the block a wool carpet
    fn is_carpet(&self) -> bool;
}

impl ItemKindExt for ItemKind {
    fn to_colored(self, color: TeamColor) -> ItemKind {
        if self.is_wool() {
            color.wool_block()
        } else if self.is_carpet() {
            color.wool_carpet()
        } else if self.is_bed() {
            color.bed_block()
        } else {
            self
        }
    }

    fn is_bed(&self) -> bool {
        matches!(
            self,
            ItemKind::WhiteBed
                | ItemKind::OrangeBed
                | ItemKind::MagentaBed
                | ItemKind::LightBlueBed
                | ItemKind::YellowBed
                | ItemKind::LimeBed
                | ItemKind::PinkBed
                | ItemKind::GrayBed
                | ItemKind::LightGrayBed
                | ItemKind::CyanBed
                | ItemKind::PurpleBed
                | ItemKind::BlueBed
                | ItemKind::BrownBed
                | ItemKind::GreenBed
                | ItemKind::RedBed
                | ItemKind::BlackBed
        )
    }

    fn is_carpet(&self) -> bool {
        matches!(
            self,
            ItemKind::WhiteCarpet
                | ItemKind::OrangeCarpet
                | ItemKind::MagentaCarpet
                | ItemKind::LightBlueCarpet
                | ItemKind::YellowCarpet
                | ItemKind::LimeCarpet
                | ItemKind::PinkCarpet
                | ItemKind::GrayCarpet
                | ItemKind::LightGrayCarpet
                | ItemKind::CyanCarpet
                | ItemKind::PurpleCarpet
                | ItemKind::BlueCarpet
                | ItemKind::BrownCarpet
                | ItemKind::GreenCarpet
                | ItemKind::RedCarpet
                | ItemKind::BlackCarpet
        )
    }

    fn is_wool(&self) -> bool {
        matches!(
            self,
            ItemKind::WhiteWool
                | ItemKind::OrangeWool
                | ItemKind::MagentaWool
                | ItemKind::LightBlueWool
                | ItemKind::YellowWool
                | ItemKind::LimeWool
                | ItemKind::PinkWool
                | ItemKind::GrayWool
                | ItemKind::LightGrayWool
                | ItemKind::CyanWool
                | ItemKind::PurpleWool
                | ItemKind::BlueWool
                | ItemKind::BrownWool
                | ItemKind::GreenWool
                | ItemKind::RedWool
                | ItemKind::BlackWool
        )
    }
}
