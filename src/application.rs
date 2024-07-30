use std::borrow::Cow;
use std::thread;
#[cfg(not(web_platform))]
use std::time;

use ::tracing::{info, warn};
#[cfg(web_platform)]
use web_time as time;
use winit::application::ApplicationHandler;
use winit::event::{KeyEvent, StartCause, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

use crate::model::Model;
use crate::model_renderer::ModelRenderer;
use crate::uniform_state::UniformState;

pub struct ApplicationFlow<'a> {
    close_requested: bool,
    window: &'a Window,
    // wgpu resources
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    shader: wgpu::ShaderModule,
    pipeline_layout: wgpu::PipelineLayout,
    config: wgpu::SurfaceConfiguration,
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    model: Model,
    model_renderer: ModelRenderer,
    uniform_layout: wgpu::BindGroupLayout,
    uniform_bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
    uniform_state: UniformState,
}

impl<'a> ApplicationFlow<'a> {
    pub async fn new(window: &'a Window) -> ApplicationFlow<'a> {
        let mut size = window.inner_size();
        size.width = size.width.max(1);
        size.height = size.height.max(1);

        let instance = wgpu::Instance::default();

        let surface: wgpu::Surface<'a> = instance.create_surface(window).unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                // Request an adapter which can render to our surface
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Failed to find an appropriate adapter");
        info!("chosen adapter is {:?}", adapter.get_info());

        // Create the logical device and command queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                    required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                        .using_resolution(adapter.limits()),
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None,
            )
            .await
            .expect("Failed to create device");

        // Load the shaders from disk
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let swapchain_format = swapchain_capabilities.formats[0];

        let config = surface
            .get_default_config(&adapter, size.width, size.height)
            .unwrap();
        surface.configure(&device, &config);

        // Bindings
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let general_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("General Buffer"),
            size: 64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let general_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("general_bind_group_layout"),
            });
        let general_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &general_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: general_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        // Load the model
        let model_bytes = include_bytes!("../assets/anime/source/Kaede_T4_9922.glb");
        let model = Model::from_bytes(
            &device,
            &queue,
            &texture_bind_group_layout,
            model_bytes,
            "Kaede_T4_9922",
        )
        .expect("Failed to load model");

        let model_renderer = ModelRenderer::new(
            &device,
            swapchain_format.into(),
            &texture_bind_group_layout,
            &general_bind_group_layout,
        );

        Self {
            close_requested: false,
            window,
            instance,
            adapter,
            shader,
            pipeline_layout,
            config,
            surface,
            device,
            queue,
            model,
            model_renderer,
            uniform_layout: general_bind_group_layout,
            uniform_bind_group: general_bind_group,
            uniform_buffer: general_buffer,
            uniform_state: UniformState::default(),
        }
    }
}

impl<'a> ApplicationHandler for ApplicationFlow<'a> {
    fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: StartCause) {
        // do nothing
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        // Have the closure take ownership of the resources.
        // `event_loop.run` never returns, therefore we must do this to ensure
        // the resources are properly cleaned up.
        let _ = (
            &self.instance,
            &self.adapter,
            &self.shader,
            &self.pipeline_layout,
        );

        match event {
            WindowEvent::KeyboardInput {
                device_id,
                event,
                is_synthetic,
            } => {
                if is_synthetic {
                    return;
                }

                // Move the model with WASD
                if event.physical_key == PhysicalKey::Code(KeyCode::KeyW) {
                    self.uniform_state.position[2] -= 0.1;
                } else if event.physical_key == PhysicalKey::Code(KeyCode::KeyS) {
                    self.uniform_state.position[2] += 0.1;
                } else if event.physical_key == PhysicalKey::Code(KeyCode::KeyA) {
                    self.uniform_state.position[0] -= 0.1;
                } else if event.physical_key == PhysicalKey::Code(KeyCode::KeyD) {
                    self.uniform_state.position[0] += 0.1;
                }

                // And Space, Shift to move up and down
                if event.physical_key == PhysicalKey::Code(KeyCode::Space) {
                    self.uniform_state.position[1] += 0.1;
                } else if event.physical_key == PhysicalKey::Code(KeyCode::ShiftLeft) {
                    self.uniform_state.position[1] -= 0.1;
                }
            }
            WindowEvent::Resized(new_size) => {
                // Reconfigure the surface with the new size
                info!("resized to {:?}", new_size);
                self.config.width = new_size.width.max(1);
                self.config.height = new_size.height.max(1);
                self.surface.configure(&self.device, &self.config);
                // On macos the window needs to be redrawn manually after resizing
                self.window.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                let frame = self
                    .surface
                    .get_current_texture()
                    .expect("Failed to acquire next swap chain texture");
                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                self.queue.write_buffer(
                    &self.uniform_buffer,
                    0,
                    bytemuck::cast_slice(&[self.uniform_state]),
                );

                let mut encoder = self
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
                {
                    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });

                    // Render the model
                    self.model_renderer.render(&mut rpass);
                    rpass.set_bind_group(1, &self.uniform_bind_group, &[]);
                    self.model.render(&mut rpass);
                }

                self.queue.submit(Some(encoder.finish()));
                frame.present();
            }
            WindowEvent::CloseRequested => {
                self.close_requested = true;
            }
            _ => {}
        };
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.window.request_redraw();
        if self.close_requested {
            event_loop.exit();
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        info!("resumed");
    }
}
