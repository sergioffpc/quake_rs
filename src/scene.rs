use std::{error::Error, time::Duration};

use crate::{
    animation::{Animation, KeyframeAnimationComponent},
    camera::Camera,
    entity::Entity,
    material::MaterialComponent,
    mesh::MeshComponent,
    model::{self, Mdl},
    renderer::Renderer,
    resource,
};

pub struct Scene {
    entities: Vec<Entity>,
}

impl Scene {
    pub fn load<S>(renderer: &Renderer, name: S) -> Result<Self, Box<dyn Error>>
    where
        S: AsRef<str>,
    {
        let zombie = Self::create_alias_entity(renderer.clone(), "progs/zombie.mdl")?;

        Ok(Self {
            entities: vec![zombie],
        })
    }

    pub fn update(&mut self, queue: &wgpu::Queue, time: &Duration) {
        for entity in self.entities.iter() {
            if let Some(animation_component) = entity.get_component::<KeyframeAnimationComponent>()
            {
                if let Some(mesh_component) = entity.get_component::<MeshComponent>() {
                    let vertices = animation_component.animate(time).unwrap();
                    mesh_component.update_vertex_buffer(&queue, &vertices);
                }
            }
        }
    }

    pub fn visible_entities(&self, camera: &Camera) -> &Vec<Entity> {
        &self.entities
    }

    fn create_alias_entity<S>(renderer: &Renderer, name: S) -> Result<Entity, Box<dyn Error>>
    where
        S: AsRef<str>,
    {
        let mdl = Mdl::load(name)?;
        let mut entity = Entity::new();
        let mesh_component = MeshComponent::new(
            renderer.clone(),
            mdl.num_verts as usize,
            &mdl.indices().to_vec(),
        );
        entity.add_component(mesh_component);

        let material_component = MaterialComponent::new(
            renderer,
            &renderer.entity_render_pipeline.texture_bind_group_layout,
            mdl.skin_width,
            mdl.skin_height,
        );

        let skin = mdl.skins.first().unwrap();
        material_component.update_texture_image(
            &renderer.queue,
            &resource::palette_index_to_rgba(&skin.indices(&Duration::ZERO)),
        );
        entity.add_component(material_component);

        let mut animation_component = KeyframeAnimationComponent::new();
        for keyframe in mdl.keyframes.iter() {
            match *keyframe {
                model::Keyframe::Static(ref kf) => {
                    let k =
                        kf.0.name
                            .trim_end_matches(|c: char| !c.is_alphabetic())
                            .to_string();
                    let animation = match animation_component.animations.get_mut(&k) {
                        Some(v) => v,
                        None => {
                            animation_component
                                .animations
                                .insert(k.to_owned(), Animation::new());
                            animation_component.animations.get_mut(&k).unwrap()
                        }
                    };
                    let vertices = mdl.vertices(&kf.0).to_vec();
                    animation.add_keyframe(vertices, Duration::from_millis(100));
                }
                model::Keyframe::Animated(_) => todo!(),
            }
        }

        Ok(entity)
    }
}
