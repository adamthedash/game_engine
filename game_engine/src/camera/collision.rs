use cgmath::{InnerSpace, Point3, Vector3};

use crate::{
    camera::Camera,
    data::block::BlockType,
    math::{bbox::AABB, ray::Ray},
    state::world::World,
};

/// Constant to avoid floating point weirdness
/// Helps not to get face stuck in walls
const EPSILON: f32 = 1e-3;

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
        // Eliminate blocks we won't actually hit
        .filter(|col| col.distance.powi(2) <= movement_vector.magnitude2())
        .min_by(|col1, col2| col1.distance.total_cmp(&col2.distance));

    let mut collision_returns = [None; 3];
    if let Some(col) = collisions {
        // Find which axis we've colliding along
        let axis = [0, 1, 2]
            .into_iter()
            .find(|axis| col.normal[*axis] != 0.)
            .unwrap();

        // How far along that axis is the collision
        let collision_distance = (-col.ray.direction * col.distance)[axis];

        collision_returns[axis] = Some(collision_distance);

        // Adjust the movement vector appropriately
        if movement_vector[axis] > 0. && movement_vector[axis] >= collision_distance {
            movement_vector[axis] = (collision_distance - EPSILON).max(0.);
        } else if movement_vector[axis] < 0. && movement_vector[axis] <= collision_distance {
            movement_vector[axis] = (collision_distance + EPSILON).min(0.);
        }

        // Re-do collision detection with new movement vector
        // TODO: There's probably a more efficient way than recursively doing this, but it should
        // only happen to a maximum depth of 3 (once for each axis)
        let (mv, c) = predict_collisions(camera, world, movement_vector);
        movement_vector = mv;
        collision_returns.iter_mut().zip(c).for_each(|(c1, c2)| {
            if c2.is_some() {
                assert!(c1.is_none());
                *c1 = c2
            }
        });
    }

    (movement_vector, collision_returns)
}
