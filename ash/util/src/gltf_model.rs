#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct Vertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    /*
    pub normal: [f32; 3],
    pub tangent: [f32; 4],
    pub bitangent: [f32; 4],
    */
}

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

pub struct GltfModel {
    pub meshes: Vec<Mesh>,
}

impl GltfModel {
    pub fn new(model_path: &std::path::Path) -> GltfModel {
        let mut meshes = vec![];
        let (gltf, buffers, _ /*images*/) = gltf::import(model_path).unwrap();
        for mesh in gltf.meshes() {
            for primitive in mesh.primitives() {
                let (vertices, indices) = load_primitive(&primitive, &buffers);
                meshes.push(Mesh { vertices, indices });
            }
        }

        GltfModel { meshes }
    }
}

fn load_primitive(primitive: &gltf::Primitive, buffers: &[gltf::buffer::Data]) -> (Vec<Vertex>, Vec<u32>) {
    let reader = primitive.reader(|buffer_data| Some(&buffers[buffer_data.index()]));
    let positions: Vec<[f32; 3]> = reader.read_positions().unwrap().into_iter().collect();
    let uvs: Vec<[f32; 2]> = reader.read_tex_coords(0).unwrap().into_f32().collect();
    /*
    let normals: Vec<[f32; 3]> = reader.read_normals().unwrap().into_iter().collect();
    let tangents: Vec<[f32; 4]> = reader.read_tangents().unwrap().into_iter().collect();
    */
    assert_eq!(positions.len(), uvs.len());
    let mut vertices: Vec<Vertex> = Vec::new();

    for (i, _) in positions.iter().enumerate() {
        vertices.push(Vertex {
            position: positions[i],
            uv: uvs[i],
        });
    }

    let indices: Vec<u32> = reader.read_indices().unwrap().into_u32().collect();

    (vertices, indices)
}
