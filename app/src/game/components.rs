use ::std::default::Default;

use crate::{
    engine::{ecs, time::Time},
    util::math::{Mat4, Vec3},
};

pub struct Timestamp {
    pub t: Time,
}

impl Default for Timestamp {
    fn default() -> Self {
        Timestamp {
            t: Time::from_millis(0.0),
        }
    }
}

impl ecs::Component for Timestamp {
    type Container = ecs::Singleton<Self>;
}

pub struct Spatial {
    pub position: Vec3,
    pub rotation: Vec3,
    pub scaling: Vec3,
}

impl Default for Spatial {
    fn default() -> Self {
        Spatial {
            position: Vec3::new(),
            rotation: Vec3::new(),
            scaling: Vec3::with(1.0, 1.0, 1.0),
        }
    }
}

impl ecs::Component for Spatial {
    type Container = ecs::BTreeComponentMap<Self>;
}

pub struct ModelMatrix {
    pub model: Mat4,
}

impl Default for ModelMatrix {
    fn default() -> Self {
        ModelMatrix {
            model: Mat4::identity(),
        }
    }
}

impl ecs::Component for ModelMatrix {
    type Container = ecs::BTreeComponentMap<Self>;
}
