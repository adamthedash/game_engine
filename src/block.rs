use cgmath::{ElementWise, EuclideanSpace, One, Quaternion};
use num_traits::ToPrimitive;

use crate::{
    bbox::AABB,
    render::state::Instance,
    world::{BlockPos, BlockType},
};

#[derive(Debug)]
pub struct Block {
    pub world_pos: BlockPos,
    pub block_type: BlockType,
}

impl Block {
    pub fn to_instance(&self) -> Instance {
        Instance {
            pos: self.world_pos.0.to_vec().cast().unwrap(),
            rotation: Quaternion::one(),
            texture_index: self.block_type.to_u32().unwrap(),
        }
    }

    /// Return the axis-aligned bounding box for this block
    pub fn aabb(&self) -> AABB<i32> {
        let p0 = self.world_pos.0;
        let p1 = p0.add_element_wise(1);
        AABB::new(&p0, &p1)
    }
}
