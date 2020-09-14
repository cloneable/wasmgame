use crate::game::math;

pub struct Camera {
    position: math::Vec3,
    target: math::Vec3,
    up: math::Vec3,

    fov: f32,
    aspect: f32,
    near: f32,
    far: f32,

    view: math::Mat4,
    projection: math::Mat4,
}

impl Camera {
    pub fn new() -> Self {
        Camera {
            position: math::Vec3::new(0.0, 0.0, 1.0),
            target: math::Vec3::new(0.0, 0.0, 0.0),
            up: math::Vec3::new(0.0, 1.0, 0.0),
            fov: 90.0,
            aspect: 1.0,
            near: 0.1,
            far: 1000.0,
            view: math::Mat4::new(),
            projection: math::Mat4::new(),
        }
    }

    pub fn set_position(&mut self, x: f32, y: f32, z: f32) {
        self.position = math::Vec3::new(x, y, z);
    }

    pub fn set_target(&mut self, x: f32, y: f32, z: f32) {
        self.target = math::Vec3::new(x, y, z);
    }

    pub fn set_frustrum(&mut self, fov: f32, aspect: f32, near: f32, far: f32) {
        self.fov = fov;
        self.aspect = aspect;
        self.near = near;
        self.far = far;
    }

    pub fn view_matrix(&self) -> &math::Mat4 {
        &self.view
    }

    pub fn projection_matrix(&self) -> &math::Mat4 {
        &self.projection
    }

    pub fn refresh(&mut self) {
        self.view = math::look_at(&self.position, &self.target, &self.up);
        self.projection = math::project(self.fov, self.aspect, self.near, self.far);
    }
}
