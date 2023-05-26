use std::error::Error;

use async_std::task;
use cgmath::{Matrix4, SquareMatrix};
use wgpu::util::DeviceExt;
use winit::window::Window;

use crate::{camera::Camera, entity::Entity, pipeline::EntityPipeline};

const TARGET_VERTICES: [Vertex1XY1UV; 6] = [
    Vertex1XY1UV {
        position: [0.0, 0.0],
        texcoord: [0.0, 1.0],
    },
    Vertex1XY1UV {
        position: [0.0, 1.0],
        texcoord: [0.0, 0.0],
    },
    Vertex1XY1UV {
        position: [1.0, 1.0],
        texcoord: [1.0, 0.0],
    },
    Vertex1XY1UV {
        position: [0.0, 0.0],
        texcoord: [0.0, 1.0],
    },
    Vertex1XY1UV {
        position: [1.0, 1.0],
        texcoord: [1.0, 0.0],
    },
    Vertex1XY1UV {
        position: [1.0, 0.0],
        texcoord: [1.0, 1.0],
    },
];

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex1XY1UV {
    pub position: [f32; 2],
    pub texcoord: [f32; 2],
}

impl Vertex1XY1UV {
    const VERTEX_ATTRS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2];

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::VERTEX_ATTRS,
        }
    }
}

pub struct Renderer {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub entity_render_pipeline: EntityPipeline,

    surface: wgpu::Surface,

    view_projection_matrix_buffer: wgpu::Buffer,
    view_projection_bind_group: wgpu::BindGroup,

    target_render_pipeline: wgpu::RenderPipeline,
    target_bind_group: wgpu::BindGroup,
    target_vertex_buffer: wgpu::Buffer,
}

impl Renderer {
    pub fn new(window: &Window) -> Result<Self, Box<dyn Error>> {
        let size = window.inner_size();

        // Create an instance and adapter
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });
        let surface = unsafe { instance.create_surface(&window) }?;
        let adapter = task::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        }))
        .unwrap();

        // Create the device and queue
        let (device, queue) = task::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
            },
            None,
        ))?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .filter(|f| f.is_srgb())
            .next()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let view_projection_matrix: [[f32; 4]; 4] = Matrix4::identity().into();
        let view_projection_matrix_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&[view_projection_matrix]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
        let view_projection_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                label: None,
            });
        let view_projection_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &view_projection_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: view_projection_matrix_buffer.as_entire_binding(),
            }],
            label: None,
        });

        let entity_render_pipeline =
            EntityPipeline::new(&device, &config, &[&view_projection_bind_group_layout]);

        let target_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let target_bind_group_layout =
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
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Depth,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: None,
            });
        let target_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &target_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &entity_render_pipeline.albedo_view,
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(
                        &entity_render_pipeline.normal_view,
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(
                        &entity_render_pipeline.depth_view,
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&target_sampler),
                },
            ],
            label: None,
        });
        let target_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&TARGET_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let target_shader = device.create_shader_module(wgpu::include_wgsl!("target.wgsl"));
        let target_render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&target_bind_group_layout],
                push_constant_ranges: &[],
            });
        let target_render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&target_render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &target_shader,
                    entry_point: "vs_main",
                    buffers: &[Vertex1XY1UV::desc()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &target_shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Cw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            });

        Ok(Self {
            device,
            queue,
            config,
            entity_render_pipeline,

            surface,

            view_projection_matrix_buffer,
            view_projection_bind_group,

            target_bind_group,
            target_vertex_buffer,
            target_render_pipeline,
        })
    }

    pub fn render(&self, camera: &Camera, entities: &Vec<Entity>) -> Result<(), Box<dyn Error>> {
        let view_projection_matrix: [[f32; 4]; 4] = camera.view_projection_matrix().into();
        self.queue.write_buffer(
            &self.view_projection_matrix_buffer,
            0,
            bytemuck::cast_slice(&[view_projection_matrix]),
        );

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        self.entity_render_pipeline.render_pass(
            &self.queue,
            &mut encoder,
            &[&self.view_projection_bind_group],
            entities,
        );

        let output = self.surface.get_current_texture().unwrap();
        let output_view = output.texture.create_view(&Default::default());
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &output_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
            render_pass.set_pipeline(&self.target_render_pipeline);
            render_pass.set_bind_group(0, &self.target_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.target_vertex_buffer.slice(..));
            render_pass.draw(0..6, 0..1);
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
