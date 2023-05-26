use cgmath::{Deg, Matrix4, Vector3, Zero};

pub struct Camera {
    pub position: Vector3<f32>,
    pub pitch: Deg<f32>,
    pub roll: Deg<f32>,
    pub yaw: Deg<f32>,

    projection: Matrix4<f32>,
}

impl Camera {
    pub fn new(fovy: Deg<f32>, aspect: f32) -> Self {
        Self {
            position: Vector3::zero(),
            pitch: Deg::zero(),
            roll: Deg::zero(),
            yaw: Deg::zero(),

            projection: cgmath::perspective(fovy, aspect, 4.0, 4096.0),
        }
    }

    pub fn view_projection_matrix(&self) -> Matrix4<f32> {
        let translation = Matrix4::from_translation(-Vector3::new(
            -self.position.y,
            self.position.z,
            -self.position.x,
        ));
        let orientation = Matrix4::from_angle_z(-self.roll)
            * Matrix4::from_angle_x(self.pitch)
            * Matrix4::from_angle_y(-self.yaw);
        let view = orientation * translation;

        self.projection * view
    }
}
