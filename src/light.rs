#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Default)]
pub struct LightingUniform {
    position: [f32; 3],
    _1: u32, // padding
    color: [f32; 3],
    _2: u32,
}

impl LightingUniform {
    pub fn new(position: [f32; 3], color: [f32; 3]) -> Self {
        Self {
            position,
            color,
            ..Default::default()
        }
    }
}
