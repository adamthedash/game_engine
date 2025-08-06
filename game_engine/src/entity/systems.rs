use std::time::Duration;

use cgmath::{Rotation, Vector3};
use hecs::World;

use crate::entity::components::{Orientation, Position};

pub trait System {
    fn tick(eccs: &mut World, duration: &Duration);
}

pub struct MoveSystem;

impl System for MoveSystem {
    fn tick(ecs: &mut World, duration: &Duration) {
        for (_, (position, orientation)) in ecs.query_mut::<(&mut Position, &mut Orientation)>() {
            let facing = orientation.0.rotate_vector(Vector3::unit_z());
            let movement_speed = 0.1 * duration.as_secs_f32();

            position.0.0 += facing * movement_speed;
        }
    }
}
