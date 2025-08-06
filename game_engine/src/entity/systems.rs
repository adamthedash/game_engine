use std::time::Duration;

use cgmath::{Rotation, Vector3};

use crate::entity::components::{Orientation, Position};

/// Basic system used to move entities around
pub fn move_system(
    positions: &mut [Position],
    orientations: &mut [Orientation],
    duration: &Duration,
) {
    positions
        .iter_mut()
        .zip(orientations)
        .for_each(|(position, orientation)| {
            let facing = orientation.0.rotate_vector(Vector3::unit_z());
            let movement_speed = 0.1 * duration.as_secs_f32();

            position.0.0 += facing * movement_speed;
        });
}
