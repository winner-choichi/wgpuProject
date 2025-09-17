// src/renderer/mesh.rs
use crate::renderer::vertex::Vertex; // Make sure to import your Vertex struct
use wgpu::util::DeviceExt;

pub struct Mesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
}

impl Mesh {
    // This function creates a new sphere mesh and immediately uploads its data to the GPU.
    pub fn new_sphere(device: &wgpu::Device, latitudes: u32, longitudes: u32, radius: f32) -> Self {
        let (vertices, indices) = generate_uv_sphere(latitudes, longitudes, radius);

        // Create a new buffer on the GPU and copy the vertex data into it.
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sphere Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // Create a new buffer on the GPU and copy the index data into it.
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sphere Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            vertex_buffer,
            index_buffer,
            num_indices: indices.len() as u32,
        }
    }
}

/// Generates a simple UV sphere with latitude/longitude subdivisions.
fn generate_uv_sphere(latitudes: u32, longitudes: u32, radius: f32) -> (Vec<Vertex>, Vec<u32>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let color = glam::Vec3::new(0.0, 0.0, 1.0); // Electron = blue

    for lat in 0..=latitudes {
        let theta = lat as f32 * std::f32::consts::PI / latitudes as f32;
        let sin_theta = theta.sin();
        let cos_theta = theta.cos();

        for lon in 0..=longitudes {
            let phi = lon as f32 * 2.0 * std::f32::consts::PI / longitudes as f32;
            let sin_phi = phi.sin();
            let cos_phi = phi.cos();

            let x = cos_phi * sin_theta;
            let y = cos_theta;
            let z = sin_phi * sin_theta;

            vertices.push(Vertex {
                position: (glam::Vec3::new(x, y, z) * radius).into(),
                color: color.into(),
            });
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
