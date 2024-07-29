use anyhow::*;
use goth_gltf::{default_extensions::Extensions, Gltf};

pub struct Model {
    pub label: String,
    pub model: Gltf<Extensions>,
}

impl Model {
    pub fn from_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        label: &str,
    ) -> Result<Self> {
        let (gltf, _): (
            goth_gltf::Gltf<goth_gltf::default_extensions::Extensions>,
            _,
        ) = goth_gltf::Gltf::from_bytes(&bytes)?;

        // Print all the useful information about the glTF model
        println!("Model: {:#?}", gltf);

        Ok(Self {
            label: label.to_string(),
            model: gltf,
        })
    }
}
