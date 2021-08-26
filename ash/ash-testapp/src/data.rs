#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub color: [f32; 3],
    pub tex_coord: [f32; 2],
}

pub const VERTICES_DATA: [Vertex; 8] = [
    Vertex {
        pos: [-0.75, -0.75, -0.5],
        color: [1.0, 0.0, 0.0],
        tex_coord: [0.0, 0.0],
    },
    Vertex {
        pos: [0.75, -0.75, -0.5],
        color: [0.0, 1.0, 0.0],
        tex_coord: [1.0, 0.0],
    },
    Vertex {
        pos: [0.75, 0.75, -0.5],
        color: [0.0, 0.0, 1.0],
        tex_coord: [1.0, 1.0],
    },
    Vertex {
        pos: [-0.75, 0.75, -0.5],
        color: [1.0, 1.0, 1.0],
        tex_coord: [0.0, 1.0],
    },
    Vertex {
        pos: [0.5, 0.5, -0.4],
        color: [1.0, 0.0, 0.0],
        tex_coord: [0.0, 0.0],
    },
    Vertex {
        pos: [1.75, 0.5, -0.4],
        color: [0.0, 1.0, 0.0],
        tex_coord: [1.0, 0.0],
    },
    Vertex {
        pos: [1.75, 1.75, -0.4],
        color: [0.0, 0.0, 1.0],
        tex_coord: [1.0, 1.0],
    },
    Vertex {
        pos: [0.5, 1.75, -0.4],
        color: [1.0, 1.0, 1.0],
        tex_coord: [0.0, 1.0],
    },
];

pub const INDICES_DATA: [u32; 12] = [0, 1, 2, 2, 3, 0, 4, 5, 6, 6, 7, 4];

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct WVPMatrices {
    pub world: cgmath::Matrix4<f32>,
    pub view: cgmath::Matrix4<f32>,
    pub projection: cgmath::Matrix4<f32>,
}
