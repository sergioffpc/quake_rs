use cgmath::{Matrix4, SquareMatrix};
use wgpu::util::DeviceExt;

use crate::{
    entity::Entity,
    material::MaterialComponent,
    mesh::{MeshComponent, Vertex1XYZ1N1UV},
    transform::TransformComponent,
};

pub struct EntityPipeline {
    pub albedo_texture: wgpu::Texture,
    pub albedo_view: wgpu::TextureView,
    pub normal_texture: wgpu::Texture,
    pub normal_view: wgpu::TextureView,
    pub depth_texture: wgpu::Texture,
    pub depth_view: wgpu::TextureView,
    pub texture_bind_group_layout: wgpu::BindGroupLayout,

    model_matrix_buffer: wgpu::Buffer,
    model_bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
}

impl EntityPipeline {
    pub fn new<'a>(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        bind_group_layouts: &'a [&'a wgpu::BindGroupLayout],
    ) -> Self {
        let target_size = wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };

        let albedo_texture = Self::create_attachment_texture(device, config.format, target_size);
        let albedo_view = albedo_texture.create_view(&Default::default());

        let normal_texture = Self::create_attachment_texture(device, config.format, target_size);
        let normal_view = normal_texture.create_view(&Default::default());

        let depth_texture =
            Self::create_attachment_texture(device, wgpu::TextureFormat::Depth32Float, target_size);
        let depth_view = depth_texture.create_view(&Default::default());

        let model_matrix: [[f32; 4]; 4] = Matrix4::identity().into();
        let model_matrix_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[model_matrix]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let (model_bind_group, model_bind_group_layout) =
            Self::create_model_bind_group(device, &model_matrix_buffer);

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
                label: None,
            });
        let mut chained_bind_group_layouts = bind_group_layouts.to_vec();
        chained_bind_group_layouts.push(&model_bind_group_layout);
        chained_bind_group_layouts.push(&texture_bind_group_layout);

        let render_pipeline =
            Self::create_render_pipeline(device, config.format, &chained_bind_group_layouts);

        Self {
            albedo_texture,
            albedo_view,
            normal_texture,
            normal_view,
            depth_texture,
            depth_view,

            texture_bind_group_layout,

            model_matrix_buffer,
            model_bind_group,

            render_pipeline,
        }
    }

    pub fn render_pass<'a>(
        &self,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        bind_groups: &'a [&'a wgpu::BindGroup],
        entities: &Vec<Entity>,
    ) {
        let albedo_attachment = Self::create_render_pass_color_attachment(&self.albedo_view);
        let normal_attachment = Self::create_render_pass_color_attachment(&self.normal_view);
        let color_attachments = [Some(albedo_attachment), Some(normal_attachment)];
        let render_pass_desc = Self::create_render_pass_desc(&color_attachments, &self.depth_view);
        let mut render_pass = encoder.begin_render_pass(&render_pass_desc);

        render_pass.set_pipeline(&self.render_pipeline);
        for (i, bind_group) in bind_groups.iter().enumerate() {
            render_pass.set_bind_group(i as u32, bind_group, &[]);
        }

        for entity in entities {
            let mut bind_group_index = bind_groups.len() as u32 - 1;

            let mut model_matrix: [[f32; 4]; 4] = Matrix4::identity().into();
            if let Some(transform_component) = entity.get_component::<TransformComponent>() {
                model_matrix = transform_component.transform_matrix().into();
            }
            queue.write_buffer(
                &self.model_matrix_buffer,
                0,
                bytemuck::cast_slice(&[model_matrix]),
            );
            bind_group_index += 1;
            render_pass.set_bind_group(bind_group_index, &self.model_bind_group, &[]);

            if let Some(material_component) = entity.get_component::<MaterialComponent>() {
                bind_group_index += 1;
                render_pass.set_bind_group(bind_group_index, &material_component.bind_group, &[]);
            }

            if let Some(mesh_component) = entity.get_component::<MeshComponent>() {
                render_pass.set_vertex_buffer(0, mesh_component.vertex_buffer.slice(..));
                render_pass.set_index_buffer(
                    mesh_component.index_buffer.slice(..),
                    wgpu::IndexFormat::Uint32,
                );
                render_pass.draw_indexed(0..mesh_component.index_count, 0, 0..1);
            }
        }
    }

    fn create_render_pass_color_attachment<'a>(
        view: &'a wgpu::TextureView,
    ) -> wgpu::RenderPassColorAttachment {
        wgpu::RenderPassColorAttachment {
            view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::BLUE),
                store: true,
            },
        }
    }

    fn create_render_pass_desc<'desc, 'tex>(
        color_attachments: &'desc [Option<wgpu::RenderPassColorAttachment<'tex>>],
        depth_view: &'tex wgpu::TextureView,
    ) -> wgpu::RenderPassDescriptor<'desc, 'tex> {
        wgpu::RenderPassDescriptor {
            label: None,
            color_attachments,
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        }
    }

    fn create_attachment_texture(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        size: wgpu::Extent3d,
    ) -> wgpu::Texture {
        device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        })
    }

    fn create_model_bind_group(
        device: &wgpu::Device,
        buffer: &wgpu::Buffer,
    ) -> (wgpu::BindGroup, wgpu::BindGroupLayout) {
        let model_bind_group_layout =
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
        let model_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &model_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: None,
        });

        (model_bind_group, model_bind_group_layout)
    }

    fn create_render_pipeline<'a>(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        bind_group_layouts: &'a [&'a wgpu::BindGroupLayout],
    ) -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(wgpu::include_wgsl!("entity.wgsl"));
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts,
                push_constant_ranges: &[],
            });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex1XYZ1N1UV::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[
                    Some(wgpu::ColorTargetState {
                        format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                    Some(wgpu::ColorTargetState {
                        format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                ],
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
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        })
    }
}
