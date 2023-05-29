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
        let entity = Self::create_alias_entity(renderer, "progs/knight.mdl")?;

        Ok(Self {
            entities: vec![entity],
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

        animation_component.current_animation = Some(
            animation_component
                .animations
                .keys()
                .next()
                .unwrap()
                .to_owned(),
        );
        let animation_vertices = animation_component.animate(&Duration::ZERO).unwrap();
        let mesh_component = MeshComponent::new(renderer, animation_vertices.len());

        let mut entity = Entity::new();
        entity.add_component(animation_component);
        entity.add_component(material_component);
        entity.add_component(mesh_component);

        Ok(entity)
    }
}
