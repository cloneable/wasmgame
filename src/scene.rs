extern crate js_sys;
extern crate log;
extern crate std;
extern crate wasm_bindgen;
extern crate web_sys;

use std::option::{Option::None, Option::Some};
use std::{vec, vec::Vec};

use crate::game;
use crate::math;

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

    pub fn set_position(&mut self, x: f32, y: f32, z: f32) -> &mut Self {
        self.position = math::Vec3::new(x, y, z);
        self
    }

    pub fn set_target(&mut self, x: f32, y: f32, z: f32) -> &mut Self {
        self.target = math::Vec3::new(x, y, z);
        self
    }

    pub fn set_frustum(&mut self, fov: f32, aspect: f32, near: f32, far: f32) -> &mut Self {
        self.fov = fov;
        self.aspect = aspect;
        self.near = near;
        self.far = far;
        self
    }

    pub fn refresh(&mut self) -> &mut Self {
        self.view = math::look_at(&self.position, &self.target, &self.up);
        self.projection = math::project(self.fov, self.aspect, self.near, self.far);
        self
    }

    pub fn view_matrix(&self) -> &math::Mat4 {
        &self.view
    }

    pub fn projection_matrix(&self) -> &math::Mat4 {
        &self.projection
    }
}

pub struct Model {
    pub vertices: Vec<f32>,
    pub normals: Vec<f32>,
    pub instances: Vec<Instance>,

    // TODO: make this more efficient
    pub instance_model_data: Vec<f32>,
    pub instance_normals_data: Vec<f32>,
}

impl Model {
    pub fn new(indexed_vertices: &'static [f32], indices: &'static [u8]) -> Self {
        let mut vertices: Vec<f32> = vec![0.0; indices.len() * 3];
        let mut normals: Vec<f32> = vec![0.0; indices.len() * 3];
        game::generate_buffers(indices, indexed_vertices, &mut vertices, &mut normals);
        Model {
            vertices,
            normals,
            instances: Vec::new(),
            instance_model_data: Vec::new(),
            instance_normals_data: Vec::new(),
        }
    }

    pub fn add_instance(&mut self, transform: math::Mat4) {
        self.instances.push(Instance::new(transform));
    }

    pub fn update_normals(&mut self, camera: &Camera) {
        self.instance_model_data.clear();
        self.instance_normals_data.clear();
        for instance in &mut self.instances {
            let mat_model_view = (camera.view_matrix() * &instance.model).to_3x3();
            instance.normals = match mat_model_view.invert() {
                Some(inv) => inv.transpose(),
                None => {
                    log::error!("mat_model_view not invertible");
                    mat_model_view
                }
            };
            self.instance_model_data
                .extend_from_slice(instance.model.slice());
            self.instance_normals_data
                .extend_from_slice(instance.normals.slice());
        }
    }
}

pub struct Instance {
    pub model: math::Mat4,
    pub normals: math::Mat4,
}

impl Instance {
    pub fn new(model: math::Mat4) -> Self {
        Instance {
            model: model,
            normals: math::Mat4::IDENTITY,
        }
    }
}
