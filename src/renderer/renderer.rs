use crate::platform::SurfaceProvider;
use crate::renderer::camera::{Camera, CameraUniform};
use crate::renderer::cloud::{CloudRenderer, CloudVertex};
use crate::renderer::mesh::Mesh;
use crate::renderer::vertex::Vertex;
use glam::Vec3;
use std::borrow::Cow;
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;

pub struct Renderer {
    surface: Option<wgpu::Surface<'static>>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,
    camera: Camera,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    sphere_pipeline: wgpu::RenderPipeline,
    sphere_mesh: Mesh,
    cloud_renderer: CloudRenderer,
}

impl Renderer {
    pub async fn new<T: SurfaceProvider>(
        target: &T,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let instance = if cfg!(target_arch = "wasm32") {
            wgpu::Instance::new(wgpu::InstanceDescriptor {
                backends: wgpu::Backends::GL,
                flags: wgpu::InstanceFlags::default(),
                dx12_shader_compiler: wgpu::Dx12Compiler::default(),
                gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
            })
        } else {
            wgpu::Instance::default()
        };

        let (surface, size) = target.create_surface(&instance)?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: surface.as_ref(),
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| "Failed to acquire GPU adapter".to_string())?;

        let limits = if cfg!(target_arch = "wasm32") {
            let mut web_limits = wgpu::Limits::downlevel_webgl2_defaults();
            web_limits.max_sampled_textures_per_shader_stage =
                web_limits.max_sampled_textures_per_shader_stage.max(4);
            web_limits
        } else {
            wgpu::Limits::downlevel_defaults()
        };

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Renderer Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: limits,
                },
                None,
            )
            .await?;

        let format = surface
            .as_ref()
            .map(|s| s.get_capabilities(&adapter).formats[0])
            .unwrap_or(wgpu::TextureFormat::Bgra8UnormSrgb);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        if let Some(surface) = &surface {
            surface.configure(&device, &config);
        }

        let camera = Camera {
            eye: Vec3::new(0.0, 1.8, 4.0),
            target: Vec3::ZERO,
            up: Vec3::Y,
            aspect: config.width as f32 / config.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        };

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Camera Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let sphere_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Nucleus Shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../shaders/sphere.wgsl"))),
        });

        let sphere_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Sphere Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout],
                push_constant_ranges: &[],
            });

        let sphere_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Sphere Pipeline"),
            layout: Some(&sphere_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &sphere_shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &sphere_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let sphere_mesh = Mesh::new_sphere(&device, 32, 32, 0.2, [0.95, 0.25, 0.35]);

        let cloud_renderer = CloudRenderer::new(&device, &config, &camera_bind_group_layout);

        Ok(Self {
            surface,
            device,
            queue,
            config,
            size,
            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            sphere_pipeline,
            sphere_mesh,
            cloud_renderer,
        })
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width == 0 || new_size.height == 0 {
            return;
        }

        self.size = new_size;
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.camera.aspect = self.config.width as f32 / self.config.height as f32;

        if let Some(surface) = &self.surface {
            surface.configure(&self.device, &self.config);
        }
    }

    pub fn update_cloud(&mut self, samples: &[CloudVertex]) {
        self.cloud_renderer
            .write_points(&self.device, &self.queue, samples);
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.render_with_ui(|_, _, _, _| {})
    }

    pub fn move_camera(&mut self, forward: f32, right: f32) {
        if forward == 0.0 && right == 0.0 {
            return;
        }

        let forward_dir = (self.camera.target - self.camera.eye).normalize_or_zero();
        if forward_dir.length_squared() == 0.0 {
            return;
        }

        let right_dir = forward_dir.cross(self.camera.up).normalize_or_zero();
        let mut delta = Vec3::ZERO;

        if forward.abs() > 0.0 {
            delta += forward_dir * forward;
        }

        if right.abs() > 0.0 && right_dir.length_squared() > 0.0 {
            delta += right_dir * right;
        }

        if delta.length_squared() == 0.0 {
            return;
        }

        self.camera.eye += delta;
        self.camera.target += delta;
    }

    pub fn zoom_camera(&mut self, amount: f32) {
        if amount == 0.0 {
            return;
        }

        let forward = self.camera.target - self.camera.eye;
        let distance = forward.length();
        if distance <= f32::EPSILON {
            return;
        }

        let min_distance = 0.5;
        let max_distance = self.camera.zfar * 0.95;
        let new_distance = (distance - amount).clamp(min_distance, max_distance);

        let direction = forward.normalize();
        self.camera.eye = self.camera.target - direction * new_distance;
    }

    pub fn render_with_ui<F>(&mut self, mut ui_pass: F) -> Result<(), wgpu::SurfaceError>
    where
        F: FnMut(&wgpu::Device, &wgpu::Queue, &mut wgpu::CommandEncoder, &wgpu::TextureView),
    {
        let surface = match &self.surface {
            Some(surface) => surface,
            None => return Ok(()),
        };

        let output = surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.camera_uniform.update_view_proj(&self.camera);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Renderer Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.04,
                            g: 0.05,
                            b: 0.09,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.sphere_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.sphere_mesh.vertex_buffer.slice(..));
            render_pass.set_index_buffer(
                self.sphere_mesh.index_buffer.slice(..),
                wgpu::IndexFormat::Uint32,
            );
            render_pass.draw_indexed(0..self.sphere_mesh.index_count, 0, 0..1);

            self.cloud_renderer
                .draw(&mut render_pass, &self.camera_bind_group);
        }

        ui_pass(&self.device, &self.queue, &mut encoder, &view);

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn size(&self) -> PhysicalSize<u32> {
        self.size
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn surface_config(&self) -> &wgpu::SurfaceConfiguration {
        &self.config
    }
}
