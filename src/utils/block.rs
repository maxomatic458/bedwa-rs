// pub trait BlockExt {
//     /// If the block is collidable
//     fn is_collidable(&self) -> bool;
// }

// impl BlockExt for Block {
//     fn is_collidable(&self) -> bool {
//         if self.state.is_air() {
//             return false;
//         }

//         match self.state {
//             BlockState::WATER | BlockState::LAVA => false,
//             _ => true,
//         }
//     }
// }
