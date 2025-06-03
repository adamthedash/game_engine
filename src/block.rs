use cgmath::{Deg, Quaternion, Rotation3, Vector3};

use crate::render::{Instance, Vertex};

pub const BLOCK_VERTICES: &[Vertex] = &[
    Vertex {
        position: [0., 0., 0.],
        texture_coords: [0., 1.],
    },
    Vertex {
        position: [1., 0., 0.],
        texture_coords: [1., 1.],
    },
    Vertex {
        position: [0., 1., 0.],
        texture_coords: [0., 0.],
    },
    Vertex {
        position: [1., 1., 0.],
        texture_coords: [1., 0.],
    },
];

pub const BLOCK_INDICES: &[u16] = &[
    0, 1, 3, //
    0, 3, 2, //
];

pub struct Block {
    pub world_pos: (i32, i32, i32),
}

impl Block {
    /// Creates renderable faces for the block
    pub fn to_instances(&self) -> [Instance; 6] {
        let world_pos = Vector3 {
            x: self.world_pos.0 as f32,
            y: self.world_pos.1 as f32,
            z: self.world_pos.2 as f32,
        };
        [
            Instance {
                pos: world_pos + Vector3::unit_z(),
                rotation: Quaternion::from_axis_angle(Vector3::unit_y(), Deg(0.)),
            },
            Instance {
                pos: world_pos + Vector3::unit_x() + Vector3::unit_z(),
                rotation: Quaternion::from_axis_angle(Vector3::unit_y(), Deg(90.)),
            },
            Instance {
                pos: world_pos + Vector3::unit_x(),
                rotation: Quaternion::from_axis_angle(Vector3::unit_y(), Deg(180.)),
            },
            Instance {
                pos: world_pos,
                rotation: Quaternion::from_axis_angle(Vector3::unit_y(), Deg(270.)),
            },
            Instance {
                pos: world_pos + Vector3::unit_z() + Vector3::unit_y(),
                rotation: Quaternion::from_axis_angle(Vector3::unit_x(), Deg(270.)),
            },
            Instance {
                pos: world_pos,
                rotation: Quaternion::from_axis_angle(Vector3::unit_x(), Deg(90.)),
            },
        ]
    }
}
