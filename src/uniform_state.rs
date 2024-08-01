use crate::camera::CameraUniform;

#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Default)]
pub struct UniformState {
    pub camera: CameraUniform,
    pub is_srgb: f32,
}
