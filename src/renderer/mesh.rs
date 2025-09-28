use crate::renderer::vertex::Vertex;
use glam::Vec3;
use wgpu::util::DeviceExt;

pub struct Mesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
}

impl Mesh {
    pub fn new_sphere(
        device: &wgpu::Device,
        latitudes: u32,
        longitudes: u32,
        radius: f32,
        color: [f32; 3],
    ) -> Self {
        let (vertices, indices) = generate_uv_sphere(latitudes, longitudes, radius, color);
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sphere Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sphere Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
        }
    }
}

fn generate_uv_sphere(
    latitudes: u32,
    longitudes: u32,
    radius: f32,
    color: [f32; 3],
) -> (Vec<Vertex>, Vec<u32>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for lat in 0..=latitudes {
        let theta = lat as f32 * std::f32::consts::PI / latitudes as f32;
        let sin_theta = theta.sin();
        let cos_theta = theta.cos();

        for lon in 0..=longitudes {
            let phi = lon as f32 * 2.0 * std::f32::consts::PI / longitudes as f32;
            let sin_phi = phi.sin();
            let cos_phi = phi.cos();

            let position = Vec3::new(cos_phi * sin_theta, cos_theta, sin_phi * sin_theta) * radius;
            vertices.push(Vertex::new(position.to_array(), color));
        }
    }

    for lat in 0..latitudes {
        for lon in 0..longitudes {
            let first = lat * (longitudes + 1) + lon;
            let second = first + longitudes + 1;

            indices.extend_from_slice(&[first, second, first + 1, second, second + 1, first + 1]);
        }
    }

    (vertices, indices)
}
