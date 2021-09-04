#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct Vertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    pub normal: [f32; 3],
}

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

pub struct Texture {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

pub struct Material {
    pub base_texture: Texture,
}

pub struct GltfModel {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
}

impl GltfModel {
    // This is a very inadequate representation of gltf but good enough for now.

    pub fn new(model_path: &std::path::Path) -> GltfModel {
        let mut meshes = vec![];
        let (gltf, buffers, images) = gltf::import(model_path).unwrap();
        for mesh in gltf.meshes() {
            for primitive in mesh.primitives() {
                let (vertices, indices) = load_primitive(&primitive, &buffers);
                meshes.push(Mesh { vertices, indices });
            }
        }

        let mut materials = vec![];
        for material in gltf.materials() {
            let base_texture_index = material.pbr_metallic_roughness().base_color_texture().unwrap().texture().index();
            let image = &images[base_texture_index];
            let mut data = vec![];
            if image.format == gltf::image::Format::R8G8B8 {
                for i in (0..image.pixels.len()).step_by(3) {
                    data.push(image.pixels[i + 0]);
                    data.push(image.pixels[i + 1]);
                    data.push(image.pixels[i + 2]);
                    data.push(255);
                }
            } else {
                data = image.pixels.clone();
            }
            materials.push(Material {
                base_texture: Texture {
                    data,
                    width: image.width,
                    height: image.height,
                },
            });
        }

        GltfModel { meshes, materials }
    }
}

fn load_primitive(primitive: &gltf::Primitive, buffers: &[gltf::buffer::Data]) -> (Vec<Vertex>, Vec<u32>) {
    let reader = primitive.reader(|buffer_data| Some(&buffers[buffer_data.index()]));
    let positions: Vec<[f32; 3]> = reader.read_positions().unwrap().into_iter().collect();
    let uvs: Vec<[f32; 2]> = reader.read_tex_coords(0).unwrap().into_f32().collect();
    let normals: Vec<[f32; 3]> = reader.read_normals().unwrap().into_iter().collect();
    assert_eq!(positions.len(), uvs.len());
    assert_eq!(positions.len(), normals.len());
    let mut vertices: Vec<Vertex> = Vec::new();

    for (i, _) in positions.iter().enumerate() {
        vertices.push(Vertex {
            position: positions[i],
            uv: uvs[i],
            normal: normals[i],
        });
    }

    let indices: Vec<u32> = reader.read_indices().unwrap().into_u32().collect();

    (vertices, indices)
}
