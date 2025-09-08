#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;
use wgpu::util::DeviceExt;

pub mod platform;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn start() {
    console_log::init_with_level(log::Level::Debug).expect("Couldn't initialize logger");
    console_error_panic_hook::set_once();

    // DOM이 로드될 때까지 기다림
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into().unwrap();

    // 캔버스 크기 설정 (HTML과 일치시킴)
    canvas.set_width(640);
    canvas.set_height(480);

    spawn_local(async move {
        match create_renderer(&canvas).await {
            Ok(mut renderer) => {
                log::info!("Renderer created successfully!");
                // 첫 번째 렌더링 수행
                match renderer.render() {
                    Ok(_) => log::info!("Triangle rendered successfully!"),
                    Err(e) => log::error!("Render failed: {:?}", e),
                }
            }
            Err(e) => {
                log::error!("Failed to create renderer: {:?}", e);
            }
        }
    });
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 2],
    color: [f32; 3],
}

impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

pub const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.5, -0.5],
        color: [1.0, 0.0, 0.0],
    },
    Vertex {
        position: [0.5, -0.5],
        color: [0.0, 1.0, 0.0],
    },
    Vertex {
        position: [0.0, 0.5],
        color: [0.0, 0.0, 1.0],
    },
];

pub const VERT_SHADER: &str = r#"
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4<f32>(input.position, 0.0, 1.0);
    out.color = input.color;
    return out;
}
"#;

pub const FRAG_SHADER: &str = r#"
struct FragmentInput {
    @location(0) color: vec3<f32>,
};

@fragment
fn fs_main(input: FragmentInput) -> @location(0) vec4<f32> {
    return vec4<f32>(input.color, 1.0);
}
"#;

pub struct Renderer<'a> {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: Option<wgpu::Surface<'a>>,
    pub config: wgpu::SurfaceConfiguration,
    pub render_pipeline: wgpu::RenderPipeline,
    pub vertex_buffer: wgpu::Buffer,
}

impl<'a> Renderer<'a> {
    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        if let Some(surface) = &self.surface {
            let output = surface.get_current_texture()?;
            let view = output
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            let mut encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.1,
                                g: 0.2,
                                b: 0.3,
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                });

                render_pass.set_pipeline(&self.render_pipeline);
                render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                render_pass.draw(0..3, 0..1);
            }

            self.queue.submit(std::iter::once(encoder.finish()));
            output.present();
        }

        Ok(())
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            if let Some(surface) = &self.surface {
                surface.configure(&self.device, &self.config);
            }
        }
    }

    pub fn size(&self) -> winit::dpi::PhysicalSize<u32> {
        winit::dpi::PhysicalSize::new(self.config.width, self.config.height)
    }
}

pub async fn create_renderer<T: platform::SurfaceProvider>(
    target: &T,
) -> Result<Renderer<'static>, Box<dyn std::error::Error>> {
    // WebGL 백엔드만 사용하여 호환성 문제 회피
    let instance = if cfg!(target_arch = "wasm32") {
        wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::GL, // WebGPU 대신 WebGL만 사용
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
            ..Default::default()
        })
        .await
        .ok_or("Failed to find an appropriate adapter")?;

    // WebGL에서는 간단한 device 요청 사용
    let (device, queue) = if cfg!(target_arch = "wasm32") {
        // WebGL은 compute shader를 지원하지 않으므로 더 제한적인 limits 사용
        let webgl_limits = wgpu::Limits {
            max_compute_workgroups_per_dimension: 0, // WebGL doesn't support compute
            max_compute_workgroup_size_x: 0,
            max_compute_workgroup_size_y: 0,
            max_compute_workgroup_size_z: 0,
            max_compute_workgroup_storage_size: 0,
            max_compute_invocations_per_workgroup: 0,
            ..wgpu::Limits::downlevel_webgl2_defaults()
        };
        
        adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("WebGL Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: webgl_limits,
                },
                None,
            )
            .await
            .map_err(|e| format!("Failed to create WebGL device: {:?}", e))?
    } else {
        // 네이티브에서는 기본 설정 사용
        adapter
            .request_device(&Default::default(), None)
            .await?
    };

    let format = if let Some(surface) = &surface {
        surface.get_capabilities(&adapter).formats[0]
    } else {
        wgpu::TextureFormat::Bgra8UnormSrgb
    };

    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: wgpu::CompositeAlphaMode::Auto,
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };

    if let Some(surface) = &surface {
        surface.configure(&device, &config);
    }

    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Shader"),
        source: wgpu::ShaderSource::Wgsl(format!("{VERT_SHADER}\n{FRAG_SHADER}").into()),
    });

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(VERTICES),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Pipeline Layout"),
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader_module,
            entry_point: "vs_main",
            buffers: &[Vertex::desc()],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader_module,
            entry_point: "fs_main",
            targets: &[Some(config.format.into())],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    });

    Ok(Renderer {
        device,
        queue,
        surface,
        config,
        render_pipeline,
        vertex_buffer,
    })
}
