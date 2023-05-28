use std::error::Error;

use async_std::task;
use cgmath::{Matrix4, SquareMatrix};
use wgpu::util::DeviceExt;
use winit::window::Window;

use crate::{
    camera::Camera,
    entity::Entity,
    pipeline::{EntityPipeline, TargetPipeline},
};

pub struct Renderer {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,

    surface: wgpu::Surface,

    view_projection_matrix_buffer: wgpu::Buffer,
    view_projection_bind_group: wgpu::BindGroup,
    pub entity_render_pipeline: EntityPipeline,
    target_render_pipeline: TargetPipeline,
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
        let target_render_pipeline = TargetPipeline::new(
            &device,
            &config,
            &entity_render_pipeline.albedo_view,
            &entity_render_pipeline.normal_view,
            &entity_render_pipeline.depth_view,
        );

        Ok(Self {
            device,
            queue,
            config,
            surface,

            view_projection_matrix_buffer,
            view_projection_bind_group,

            entity_render_pipeline,
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
        self.target_render_pipeline
            .render_pass(&mut encoder, &output_view);
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
