use crate::engine::ecs;
use crate::util::math::{Mat4, Vec3};

pub struct Camera {
    pub position: Vec3,
}

impl ecs::Component for Camera {
    type Container = ecs::Singleton<Self>;
}

pub struct Spatial {
    pub position: Vec3,
    pub rotation: Vec3,
    pub scaling: Vec3,
}

impl ecs::Component for Spatial {
    type Container = ecs::BTreeComponentMap<Self>;
}

pub struct ModelMatrix {
    pub model: Mat4,
}

impl ecs::Component for ModelMatrix {
    type Container = ecs::BTreeComponentMap<Self>;
}
