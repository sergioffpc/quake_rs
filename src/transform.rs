use cgmath::{ElementWise, Matrix4, Quaternion, Rad, Rotation3, Vector3, Zero};

pub struct TransformComponent {
    position: Vector3<f32>,
    orientation: Quaternion<f32>,
    scale: Vector3<f32>,
}

impl TransformComponent {
    pub fn new() -> Self {
        Self {
            position: Vector3::zero(),
            orientation: Quaternion::zero(),
            scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }

    pub fn translate(&mut self, translation: Vector3<f32>) {
        self.position += translation;
    }

    pub fn rotate<A: Into<Rad<f32>>>(&mut self, axis: Vector3<f32>, angle: A) {
        let rotation = Quaternion::from_axis_angle(axis, angle);
        self.orientation = self.orientation * rotation;
    }

    pub fn scale(&mut self, scale: Vector3<f32>) {
        self.scale.mul_element_wise(scale);
    }

    pub fn transform_matrix(&self) -> Matrix4<f32> {
        let translation_matrix = Matrix4::from_translation(self.position);
        let rotation_matrix = Matrix4::from(self.orientation);
        let scale_matrix = Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z);

        translation_matrix * rotation_matrix * scale_matrix
    }
}
