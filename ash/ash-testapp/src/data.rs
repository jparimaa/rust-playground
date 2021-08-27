#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct WVPMatrices {
    pub world: cgmath::Matrix4<f32>,
    pub view: cgmath::Matrix4<f32>,
    pub projection: cgmath::Matrix4<f32>,
}
