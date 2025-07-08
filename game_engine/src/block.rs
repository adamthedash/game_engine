use cgmath::{ElementWise, EuclideanSpace, Matrix3, Matrix4, One};

use crate::{
    bbox::AABB,
    data::{block::BlockType, loader::BLOCKS},
    render::shaders::texture::{self, Instance},
    state::world::BlockPos,
};

#[derive(Debug)]
pub struct Block {
    pub block_pos: BlockPos,
    pub block_type: BlockType,
}

impl Block {
    #[inline]
    pub fn to_instance(&self) -> Instance {
        let texture_index = BLOCKS.get().unwrap()[self.block_type].texture_index;
        texture::Instance {
            model: Matrix4::from_translation(self.block_pos.to_world_pos().0.to_vec()).into(),
            texture_index,
            normal: Matrix3::one().into(),
        }
    }

    /// Return the axis-aligned bounding box for this block
    #[inline]
    pub fn aabb(&self) -> AABB<i32> {
        let p0 = self.block_pos.0;
        let p1 = p0.add_element_wise(1);
        AABB::new(&p0, &p1)
    }
}
