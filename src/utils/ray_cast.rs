use valence::{
    math::{Aabb, DVec3, I16Vec3, Vec3},
    BlockPos, ChunkLayer,
};

use crate::utils::aabb::AabbExt;

pub fn thick_world_raycast(
    chunk_layer: &ChunkLayer,
    mut hitbox: Aabb,
    mut direction: Vec3,
    max_distance: f64,
    step: f64,
) -> Vec<(DVec3, I16Vec3)> {
    let mut distance = 0.0;
    // let mut pos = hitbox.center();
    direction = direction.normalize();

    while distance < max_distance {
        hitbox = Aabb::new(
            hitbox.min() + direction.as_dvec3() * step,
            hitbox.max() + direction.as_dvec3() * step,
        );

        let mut cols = get_real_world_intersections(chunk_layer, &hitbox);
        if !cols.is_empty() {
            cols.sort_by(|a, b| {
                let a_dist = (a.0 - hitbox.center()).length_squared();
                let b_dist = (b.0 - hitbox.center()).length_squared();
                a_dist.partial_cmp(&b_dist).unwrap()
            });

            return cols;
        }

        distance += step;
    }

    vec![]
}

/// Returns a list of all the blocks that are inside (or intersect) the given AABB
fn aabb_full_block_intersections(aabb: &Aabb) -> Vec<BlockPos> {
    let mut blocks = Vec::new();

    let min = aabb.min().floor();
    let max = aabb.max().ceil();

    for x in (min.x as i32)..(max.x as i32) {
        for y in (min.y as i32)..(max.y as i32) {
            for z in (min.z as i32)..(max.z as i32) {
                blocks.push(BlockPos { x, y, z });
            }
        }
    }

    blocks
}

/// Returns the position of the first intersection as well as the normal of the blockface that was hit
fn get_real_world_intersections(chunk_layer: &ChunkLayer, aabb: &Aabb) -> Vec<(DVec3, I16Vec3)> {
    let block_intersections = aabb_full_block_intersections(aabb);

    let mut cols = Vec::new();

    for block_pos in block_intersections {
        let Some(block) = chunk_layer.block(block_pos) else {
            continue;
        };

        if block.state.is_air() {
            continue;
        }

        for mut block_hitbox in block.state.collision_shapes() {
            block_hitbox = Aabb::new(
                block_hitbox.min()
                    + DVec3::new(block_pos.x as f64, block_pos.y as f64, block_pos.z as f64),
                block_hitbox.max()
                    + DVec3::new(block_pos.x as f64, block_pos.y as f64, block_pos.z as f64),
            );

            if aabb.intersects(block_hitbox) {
                let hit_point = block_hitbox.projected_point_exact(aabb.center());
                let hit_normal = get_hit_normal(hit_point, block_hitbox);

                cols.push((hit_point, hit_normal));
            }
        }
    }

    cols
}

fn get_hit_normal(point: DVec3, block: Aabb) -> I16Vec3 {
    let mut normal = I16Vec3::ZERO;

    if point.x == block.min().x {
        normal.x = -1;
    } else if point.x == block.max().x {
        normal.x = 1;
    }

    if point.y == block.min().y {
        normal.y = -1;
    } else if point.y == block.max().y {
        normal.y = 1;
    }

    if point.z == block.min().z {
        normal.z = -1;
    } else if point.z == block.max().z {
        normal.z = 1;
    }

    if normal == I16Vec3::ZERO {
        panic!(
            "No normal found for point: {:?} and block: {:?}",
            point, block
        );
    }

    normal
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_block_aabb_intersections1() {
        let aabb = Aabb::new(DVec3::new(0.0, 0.0, 0.0), DVec3::new(1.0, 1.0, 1.0));
        let blocks = aabb_full_block_intersections(&aabb);

        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0], BlockPos::new(0, 0, 0));
    }

    #[test]
    fn test_block_aabb_intersections2() {
        let aabb = Aabb::new(DVec3::new(0.5, 0.5, 0.5), DVec3::new(1.5, 1.5, 1.5));
        let blocks = aabb_full_block_intersections(&aabb);

        assert_eq!(blocks.len(), 8);
        assert_eq!(blocks[0], BlockPos::new(0, 0, 0));
        assert_eq!(blocks[1], BlockPos::new(0, 0, 1));
        assert_eq!(blocks[2], BlockPos::new(0, 1, 0));
        assert_eq!(blocks[3], BlockPos::new(0, 1, 1));
        assert_eq!(blocks[4], BlockPos::new(1, 0, 0));
        assert_eq!(blocks[5], BlockPos::new(1, 0, 1));
        assert_eq!(blocks[6], BlockPos::new(1, 1, 0));
        assert_eq!(blocks[7], BlockPos::new(1, 1, 1));
    }
}
