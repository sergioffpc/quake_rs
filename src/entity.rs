use std::any::{Any, TypeId};
use std::collections::HashMap;

use crate::animation::KeyframeAnimationComponent;
use crate::material::MaterialComponent;
use crate::mesh::MeshComponent;
use crate::transform::TransformComponent;

pub enum ComponentType {
    Transform,
    Mesh,
    Material,
    KeyframeAnimation,
}

impl ComponentType {
    fn get_type_id(&self) -> TypeId {
        match self {
            ComponentType::KeyframeAnimation => TypeId::of::<KeyframeAnimationComponent>(),
            ComponentType::Material => TypeId::of::<MaterialComponent>(),
            ComponentType::Mesh => TypeId::of::<MeshComponent>(),
            ComponentType::Transform => TypeId::of::<TransformComponent>(),
        }
    }
}

pub trait Component: Any {
    fn get_type() -> ComponentType;
}

impl Component for KeyframeAnimationComponent {
    fn get_type() -> ComponentType {
        ComponentType::KeyframeAnimation
    }
}

impl Component for MaterialComponent {
    fn get_type() -> ComponentType {
        ComponentType::Material
    }
}

impl Component for MeshComponent {
    fn get_type() -> ComponentType {
        ComponentType::Mesh
    }
}

impl Component for TransformComponent {
    fn get_type() -> ComponentType {
        ComponentType::Transform
    }
}

pub struct Entity {
    components: HashMap<TypeId, Box<dyn Any>>,
}

impl Entity {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }

    pub fn add_component<T: Component>(&mut self, component: T) {
        self.components
            .insert(T::get_type().get_type_id(), Box::new(component));
    }

    pub fn get_component<T: Component>(&self) -> Option<&T> {
        self.components
            .get(&T::get_type().get_type_id())
            .map(|component| component.downcast_ref::<T>())
            .flatten()
    }
}
