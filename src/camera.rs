use std::f32::consts::FRAC_PI_2;

use cgmath::{Matrix4, Point3, Rad, Vector3, Zero};

pub struct Camera {
    field_of_view: Rad<f32>,
    aspect_ratio: f32,
    near_clip_plane: f32,
    far_clip_plane: f32,

    pub eye: Point3<f32>,
    pub center: Point3<f32>,
    pub up: Vector3<f32>,
}

impl Camera {
    pub fn new(width: i32, height: i32) -> Self {
        let field_of_view = Rad(FRAC_PI_2);
        let aspect_ratio = width as f32 / height as f32;
        let near = 4.0;
        let far = 4096.0;

        Self {
            field_of_view,
            aspect_ratio,
            near_clip_plane: near,
            far_clip_plane: far,

            eye: Point3::new(0f32, 0f32, 0.0f32),
            center: Point3::new(0f32, 0f32, 0f32),
            up: Vector3::unit_y(),
        }
    }

    pub fn view_projection_matrix(&self) -> Matrix4<f32> {
        let view_matrix = cgmath::Matrix4::look_at_rh(self.eye, self.center, self.up);
        let projection_matrix = cgmath::perspective(
            self.field_of_view,
            self.aspect_ratio,
            self.near_clip_plane,
            self.far_clip_plane,
        );

        projection_matrix * view_matrix
    }
}
