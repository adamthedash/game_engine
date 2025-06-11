use cgmath::{One, Quaternion, Vector3};
use num_traits::ToPrimitive;

use crate::{chunk::BlockType, render::Instance};

pub struct Block {
    pub world_pos: (i32, i32, i32),
    pub block_type: BlockType,
}

impl Block {
    pub fn to_instance(&self) -> Instance {
        Instance {
            pos: Vector3 {
                x: self.world_pos.0 as f32,
                y: self.world_pos.1 as f32,
                z: self.world_pos.2 as f32,
            },
            rotation: Quaternion::one(),
            texture_index: self.block_type.to_u32().unwrap(),
        }
    }
}
