use anyhow::*;
use gltf::{accessor::DataType, buffer::Data, Document, Semantic};
use wgpu::util::DeviceExt;

use crate::texture::Texture;

// Color vertex
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ColorVertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

impl ColorVertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

// Texture vertex
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TextureVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
}

impl TextureVertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

pub struct ModelMesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
    pub texture_index: Option<usize>,
}

impl ModelMesh {
    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>, textures: &'a [Texture]) {
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        let texture = self
            .texture_index
            .map(|index| &textures[index])
            .unwrap_or(&textures[0]);
        let bind_group = texture
            .bind_group
            .as_ref()
            .expect("Texture has no bind group");
        render_pass.set_bind_group(0, bind_group, &[]);
        render_pass.draw_indexed(0..self.num_elements, 0, 0..1);
    }
}

pub struct Model {
    pub label: String,
    pub model: Document,
    pub images: Vec<Texture>,
    pub meshes: Vec<ModelMesh>,
}

impl Model {
    pub fn from_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        binding_layout: &wgpu::BindGroupLayout,
        bytes: &[u8],
        label: &str,
    ) -> Result<Self> {
        let (document, buffers, images) = gltf::import_slice(&bytes)?;

        // Print all the meshes in the gltf file
        // let mut meshes = Vec::new();
        let mut mesh_models = vec![];
        for mesh in document.meshes() {
            for primitive in mesh.primitives() {
                let mut positions = Vec::new();
                let mut texture_positions = Vec::new();
                for (semantic, accessor) in primitive.attributes() {
                    // let mut position_data =
                    let view = accessor.view().context("Accessor has no view")?;
                    let offset = view.offset();
                    let length = view.length();

                    let buffer = view.buffer();
                    let index = buffer.index();
                    let buffer_data = buffers.get(index).context("Buffer not found")?;
                    let buffer_data = buffer_data.0.as_slice();
                    let buffer_data = &buffer_data[offset..offset + length];

                    match semantic {
                        Semantic::Positions => {
                            // turn into a list of float numbers
                            let float_data: Vec<f32> = buffer_data
                                .chunks_exact(4)
                                .map(|chunk| {
                                    let mut bytes = [0; 4];
                                    bytes.copy_from_slice(chunk);
                                    f32::from_le_bytes(bytes)
                                })
                                .collect();
                            positions = float_data
                                .chunks(3)
                                .map(|chunk| [chunk[0], chunk[1], chunk[2]])
                                .collect();
                        }
                        Semantic::TexCoords(0) => {
                            // turn into a list of float numbers
                            let float_data: Vec<f32> = buffer_data
                                .chunks_exact(4)
                                .map(|chunk| {
                                    let mut bytes = [0; 4];
                                    bytes.copy_from_slice(chunk);
                                    f32::from_le_bytes(bytes)
                                })
                                .collect();
                            texture_positions = float_data
                                .chunks(2)
                                .map(|chunk| [chunk[0], chunk[1]])
                                .collect();
                        }
                        _ => {}
                    }
                }

                assert_eq!(positions.len(), texture_positions.len());

                // Vertices
                let mut vertices = Vec::new();
                for (position, tex_coords) in positions.iter().zip(texture_positions.iter()) {
                    vertices.push(TextureVertex {
                        position: *position,
                        tex_coords: *tex_coords,
                    });
                }

                let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });

                // Indices
                let indices_accessor = primitive.indices().context("No indices accessor")?;
                let view = indices_accessor
                    .view()
                    .context("Indices accessor has no view")?;
                let buffer = view.buffer();
                let index = buffer.index();
                let buffer_data = buffers.get(index).context("Buffer not found")?;
                let buffer_data = buffer_data.0.as_slice();
                let buffer_data = &buffer_data[view.offset()..view.offset() + view.length()];
                let component_type = indices_accessor.data_type();
                let component_size = match component_type {
                    DataType::U16 => 2,
                    DataType::U32 => 4,
                    _ => panic!("Unsupported component type"),
                };
                let buffer_data = buffer_data
                    .chunks_exact(component_size)
                    .map(|chunk| {
                        // always read as u32
                        let mut bytes = [0; 4];
                        for (i, byte) in chunk.iter().enumerate() {
                            bytes[i] = *byte;
                        }
                        u32::from_le_bytes(bytes)
                    })
                    .collect::<Vec<_>>();

                let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: bytemuck::cast_slice(&buffer_data),
                    usage: wgpu::BufferUsages::INDEX,
                });

                // Image
                let material = primitive.material();
                let texture = material
                    .pbr_metallic_roughness()
                    .base_color_texture()
                    .map(|texture| texture.texture());
                let image = texture.map(|texture| texture.source());
                let image_index = image.map(|image| image.index());

                let num_elements = buffer_data.len() as u32;
                mesh_models.push(ModelMesh {
                    vertex_buffer,
                    index_buffer,
                    num_elements,
                    texture_index: image_index,
                });
            }
        }

        let images = images
            .into_iter()
            .map(|image| {
                Texture::from_rgba8(
                    device,
                    queue,
                    binding_layout,
                    &image.pixels,
                    (image.width, image.height),
                    None,
                )
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(Self {
            label: label.to_string(),
            model: document,
            images,
            meshes: mesh_models,
        })
    }

    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        for mesh in &self.meshes {
            mesh.render(render_pass, &self.images);
        }
    }
}
