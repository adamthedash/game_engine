use cgmath::Vector3;
use itertools::Itertools;

use crate::{
    camera::Camera,
    data::block::BlockType,
    state::world::{BlockPos, World},
    world_gen::ChunkGenerator,
};

/// The result of running collision detection for the player
pub struct Collisions {
    pub x_pos: bool,
    pub x_neg: bool,
    pub y_pos: bool,
    pub y_neg: bool,
    pub z_pos: bool,
    pub z_neg: bool,
}

/// Perform collision detection for all 6 directions around the player
/// TODO: Face-based detection, high velocity collisions
pub fn detect_collisions<G: ChunkGenerator>(camera: &Camera, world: &World<G>) -> Collisions {
    let camera_pos = camera.pos.get();
    let player_aabb = camera.aabb();

    let colliding_with = |pos: &BlockPos| {
        if let Some(block) = world.get_block(pos) {
            if block.block_type == BlockType::Air {
                false
            } else {
                player_aabb.intersects(&block.aabb().to_f32())
            }
        } else {
            false
        }
    };

    let collisions = [player_aabb.start, player_aabb.end]
        .into_iter()
        .flat_map(|point| {
            (0..3)
                .map(|axis| {
                    let mut test_point = camera_pos;
                    test_point.0[axis] = point[axis];

                    let test_block = test_point.to_block_pos();
                    colliding_with(&test_block)
                })
                .collect_array::<3>()
                .unwrap()
        })
        .collect_array::<6>()
        .unwrap();

    Collisions {
        x_neg: collisions[0],
        y_neg: collisions[1],
        z_neg: collisions[2],
        x_pos: collisions[3],
        y_pos: collisions[4],
        z_pos: collisions[5],
    }
}

/// Adjust a proposed movement vector with collision detection.  
pub fn adjust_movement_vector(
    mut movement_vector: Vector3<f32>,
    collisions: &Collisions,
) -> Vector3<f32> {
    if collisions.x_pos && movement_vector.x > 0. {
        movement_vector.x = 0.;
    }
    if collisions.x_neg && movement_vector.x < 0. {
        movement_vector.x = 0.;
    }
    if collisions.y_pos && movement_vector.y > 0. {
        movement_vector.y = 0.;
    }
    if collisions.y_neg && movement_vector.y < 0. {
        movement_vector.y = 0.;
    }
    if collisions.z_pos && movement_vector.z > 0. {
        movement_vector.z = 0.;
    }
    if collisions.z_neg && movement_vector.z < 0. {
        movement_vector.z = 0.;
    }

    movement_vector
}
