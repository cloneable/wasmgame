use ::std::{default::Default, vec::Vec};

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
    type Container = ecs::VecIndex<Self>;
}

#[derive(Default)]
pub struct ModelMatrix {
    pub model: Mat4,
}

impl ecs::Component for ModelMatrix {
    type Container = ecs::VecIndex<Self>;
}

#[derive(Default)]
pub struct Hexatile {
    pub uvw: (i16, i16, i16),
    pub active: bool,
    pub hidden: bool,

    pub changed: bool,
}

impl ecs::Component for Hexatile {
    type Container = ecs::VecIndex<Self>;
}

#[derive(Default)]
pub struct HexatileField {
    pub width: usize,
    pub data: Vec<u8>,

    pub changed: bool,
}

impl HexatileField {
    fn index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    fn coords(&self, i: usize) -> (usize, usize) {
        let x = i % self.width;
        let y = i / self.width;
        (x, y)
    }
}

impl ecs::Component for HexatileField {
    type Container = ecs::Singleton<Self>;
}
