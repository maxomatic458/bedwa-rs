use valence::ItemKind;

use crate::colors::TeamColor;

pub trait ItemKindExtColor {
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

impl ItemKindExtColor for ItemKind {
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

pub trait ItemKindExtWeapons {
    fn damage(&self) -> f32;
    fn knockback(&self) -> f32;
    fn burn_duration(&self) -> Option<u32>;
    fn use_ammo(&self) -> bool;
}

impl ItemKindExtWeapons for ItemKind {
    fn damage(&self) -> f32 {
        match self {
            ItemKind::WoodenSword => 4.0,
            ItemKind::WoodenPickaxe => 2.0,
            ItemKind::WoodenHoe => 1.0,
            ItemKind::WoodenShovel => 2.5,

            ItemKind::StoneSword => 5.0,
            ItemKind::StonePickaxe => 3.0,
            ItemKind::StoneHoe => 1.0,
            ItemKind::StoneShovel => 3.5,

            ItemKind::IronSword => 6.0,
            ItemKind::IronPickaxe => 5.0,
            ItemKind::IronHoe => 1.0,
            ItemKind::IronShovel => 2.5,

            ItemKind::GoldenSword => 4.0,
            ItemKind::GoldenPickaxe => 2.0,
            ItemKind::GoldenHoe => 1.0,
            ItemKind::GoldenShovel => 2.5,

            ItemKind::DiamondSword => 7.0,
            ItemKind::DiamondPickaxe => 5.0,
            ItemKind::DiamondHoe => 1.0,
            ItemKind::DiamondShovel => 5.5,

            ItemKind::NetheriteSword => 8.0,
            ItemKind::NetheritePickaxe => 6.0,
            ItemKind::NetheriteHoe => 1.0,
            ItemKind::NetheriteShovel => 6.5,
            _ => 0.5,
        }
    }

    fn knockback(&self) -> f32 {
        todo!()
    }

    fn burn_duration(&self) -> Option<u32> {
        todo!()
    }

    fn use_ammo(&self) -> bool {
        todo!()
    }
}
