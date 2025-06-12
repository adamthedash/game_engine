use cgmath::{ElementWise, One, Point3, Quaternion, Vector3};
use num_traits::ToPrimitive;

use crate::{
    bbox::AABB,
    chunk::{BlockType, WorldPos},
    render::Instance,
};

#[derive(Debug)]
pub struct Block {
    pub world_pos: WorldPos,
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

    /// Return the axis-aligned bounding box for this block
    pub fn aabb(&self) -> AABB<i32> {
        let p0 = Point3 {
            x: self.world_pos.0,
            y: self.world_pos.1,
            z: self.world_pos.2,
        };
        let p1 = p0.add_element_wise(1);
        AABB::new(&p0, &p1)
    }
}
