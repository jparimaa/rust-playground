#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct Vertex {
    pub pos: [f32; 4],
    pub tex_coord: [f32; 2],
}

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

pub struct ObjModel {
    pub meshes: Vec<Mesh>,
}

impl ObjModel {
    pub fn new(model_path: &std::path::Path) -> ObjModel {
        let obj_file = tobj::load_obj(model_path).expect("Failed to load obj file");

        let mut meshes: Vec<Mesh> = Vec::new();

        let (models, _) = obj_file;
        for model in models.iter() {
            let positions = &model.mesh.positions;
            let tex_coords = &model.mesh.texcoords;
            if tex_coords.len() == 0 {
                panic!("Missing texture coordinates")
            }

            let mut vertices = vec![];
            let total_vertex_count = positions.len() / 3;
            for i in 0..total_vertex_count {
                let vertex = Vertex {
                    pos: [positions[i * 3], positions[i * 3 + 1], positions[i * 3 + 2], 1.0],
                    tex_coord: [tex_coords[i * 2], tex_coords[i * 2 + 1]],
                };
                vertices.push(vertex);
            }

            meshes.push(Mesh {
                vertices,
                indices: model.mesh.indices.clone(),
            });
        }

        ObjModel { meshes }
    }
}
