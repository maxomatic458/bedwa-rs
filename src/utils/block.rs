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

use valence::{math::DVec3, BlockPos};

pub fn get_block_center(pos: BlockPos) -> DVec3 {
    let x = if pos.x < 0 {
        pos.x as f64 - 0.5
    } else {
        pos.x as f64 + 0.5
    };
    let y = pos.y as f64;
    let z = if pos.z < 0 {
        pos.z as f64 - 0.5
    } else {
        pos.z as f64 + 0.5
    };

    DVec3::new(x, y, z)
}
