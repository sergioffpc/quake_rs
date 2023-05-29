use crate::renderer::Renderer;

pub struct MeshComponent {
    pub vertex_buffer: wgpu::Buffer,
    pub vertex_count: usize,
}

impl MeshComponent {
    pub fn new(renderer: &Renderer, vertex_count: usize) -> Self {
        let vertex_buffer = renderer.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (std::mem::size_of::<Vertex>() * vertex_count) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            vertex_buffer,
            vertex_count,
        }
    }

    pub fn update_vertex_buffer(&self, queue: &wgpu::Queue, vertices: &Vec<Vertex>) {
        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(vertices));
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub texcoord: [f32; 2],
}

impl Vertex {
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
