use valence::BlockState;

pub trait BlockStateExt {
    /// Is the block state a bed
    fn is_bed(&self) -> bool;
}

impl BlockStateExt for BlockState {
    fn is_bed(&self) -> bool {
        matches!(
            self,
            &BlockState::RED_BED
                | &BlockState::ORANGE_BED
                | &BlockState::YELLOW_BED
                | &BlockState::LIME_BED
                | &BlockState::GREEN_BED
                | &BlockState::CYAN_BED
                | &BlockState::LIGHT_BLUE_BED
                | &BlockState::BLUE_BED
                | &BlockState::PURPLE_BED
                | &BlockState::MAGENTA_BED
                | &BlockState::PINK_BED
                | &BlockState::WHITE_BED
                | &BlockState::GRAY_BED
                | &BlockState::LIGHT_GRAY_BED
                | &BlockState::BROWN_BED
                | &BlockState::BLACK_BED
        )
    }
}
