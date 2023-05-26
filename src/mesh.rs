use std::rc::Rc;

use wgpu::util::DeviceExt;

use crate::renderer::Renderer;

pub struct MeshComponent {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,

    renderer: Rc<Renderer>,
}

impl MeshComponent {
    pub fn new(renderer: Rc<Renderer>, vertex_count: usize, indices: &Vec<u32>) -> Self {
        let vertex_buffer = renderer.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (std::mem::size_of::<Vertex1XYZ1N1UV>() * vertex_count) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let index_buffer = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(indices),
                usage: wgpu::BufferUsages::INDEX,
            });

        Self {
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,

            renderer: renderer.clone(),
        }
    }

    pub fn update_vertex_buffer(&self, vertices: &Vec<Vertex1XYZ1N1UV>) {
        self.renderer
            .queue
            .write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(vertices));
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex1XYZ1N1UV {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub texcoord: [f32; 2],
}

impl Vertex1XYZ1N1UV {
    const VERTEX_ATTRS: [wgpu::VertexAttribute; 3] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x2];

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::VERTEX_ATTRS,
        }
    }
}
