use cgmath::{ElementWise, EuclideanSpace, Matrix3, Matrix4, One};
use num_traits::ToPrimitive;

use crate::{
    bbox::AABB,
    render::shaders::texture::{self, Instance},
    world::{BlockPos, BlockType},
};

#[derive(Debug)]
pub struct Block {
    pub block_pos: BlockPos,
    pub block_type: BlockType,
}

impl Block {
    pub fn to_instance(&self) -> Instance {
        texture::Instance {
            model: Matrix4::from_translation(self.block_pos.to_world_pos().0.to_vec()).into(),
            texture_index: self.block_type.to_u32().unwrap(),
            normal: Matrix3::one().into(),
        }
    }

    /// Return the axis-aligned bounding box for this block
    pub fn aabb(&self) -> AABB<i32> {
        let p0 = self.block_pos.0;
        let p1 = p0.add_element_wise(1);
        AABB::new(&p0, &p1)
    }
}
