use cgmath::{ElementWise, InnerSpace, Point3, Vector3};

use crate::{
    camera::Camera,
    data::block::BlockType,
    math::{bbox::AABB, ray::Ray},
    state::world::World,
};

/// Predict what we'll collide with if we move in the given direction
/// Takes a proposed movement vector and returns an allowed one, along with any collisions.
pub fn predict_collisions(
    camera: &Camera,
    world: &World,
    mut movement_vector: Vector3<f32>,
) -> (Vector3<f32>, [Option<f32>; 3]) {
    // Not moving, so we can't hit anything
    if movement_vector.magnitude2() == 0. {
        return (movement_vector, [None; 3]);
    }

    let player_aabb = camera.aabb();

    // Expand player AABB by block size
    let test_aabb = player_aabb.minkowski_sum(&AABB::new(
        &Point3::new(-1., -1., -1.),
        &Point3::new(0., 0., 0.),
    ));

    // Coarse selection of blocks we might hit
    let movement_aabb = player_aabb
        .minkowski_sum(&AABB::new(
            &Point3::new(
                movement_vector.x.min(0.),
                movement_vector.y.min(0.),
                movement_vector.z.min(0.),
            ),
            &Point3::new(
                movement_vector.x.max(0.),
                movement_vector.y.max(0.),
                movement_vector.z.max(0.),
            ),
        ))
        .to_block_aabb();
    let candidate_blocks = movement_aabb.iter_blocks();

    let EPS = 0.01;

    // Test each block for collisions
    let collisions = candidate_blocks
        // Don't collide with air blocks
        .filter(|pos| {
            world
                .get_block(pos)
                .is_some_and(|b| b.block_type != BlockType::Air)
        })
        // Point intersection test between block and player
        .flat_map(|pos| {
            let ray = Ray::new(pos.to_world_pos().0, -movement_vector);
            test_aabb.intersect_ray(&ray)
        })
        // Collect the closest hits across the blocks
        .fold([None; 3], |mut acc: [Option<f32>; 3], collision| {
            // How far from the colliding face is the block
            let hit_vector = -collision.ray.direction * collision.distance;
            let abs_normal = collision.normal.mul_element_wise(collision.normal);
            let normal_distance = hit_vector.mul_element_wise(abs_normal);
            println!(
                "{:?} {:?} {:?}",
                collision.ray.pos, normal_distance, hit_vector
            );

            // Closest hits
            [0, 1, 2].into_iter().for_each(|axis| {
                if movement_vector[axis] > 0. && collision.normal[axis] > 0. {
                    if let Some(x) = acc[axis].as_mut() {
                        *x = x.min(hit_vector[axis] - EPS).max(0.);
                    } else {
                        acc[axis] = Some(hit_vector[axis] - EPS);
                    }
                } else if movement_vector[axis] < 0. && collision.normal[axis] < 0. {
                    if let Some(x) = acc[axis].as_mut() {
                        *x = x.max(hit_vector[axis] + EPS).min(0.);
                    } else {
                        acc[axis] = Some(hit_vector[axis] + EPS);
                    }
                }
            });

            acc
        });

    // Adjust the movement vector
    collisions.iter().enumerate().for_each(|(axis, hit)| {
        if let Some(hit) = hit {
            if movement_vector[axis] > 0. {
                movement_vector[axis] = movement_vector[axis].min(*hit);
            } else if movement_vector[axis] < 0. {
                movement_vector[axis] = movement_vector[axis].max(*hit);
            }
        }
    });

    (movement_vector, collisions)
}
