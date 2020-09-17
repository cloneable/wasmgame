extern crate js_sys;
extern crate log;
extern crate std;
extern crate wasm_bindgen;
extern crate web_sys;

use std::option::{Option::None, Option::Some};
use std::{vec, vec::Vec};

use super::math::{look_at, project, Mat4, Vec3};
use super::util;

pub struct Camera {
    position: Vec3,
    target: Vec3,
    up: Vec3,

    fov: f32,
    aspect: f32,
    near: f32,
    far: f32,

    view: Mat4,
    projection: Mat4,
}

impl Camera {
    pub fn new() -> Self {
        Camera {
            position: Vec3::new(0.0, 0.0, 1.0),
            target: Vec3::new(0.0, 0.0, 0.0),
            up: Vec3::new(0.0, 1.0, 0.0),
            fov: 90.0,
            aspect: 1.0,
            near: 0.1,
            far: 1000.0,
            view: Mat4::new(),
            projection: Mat4::new(),
        }
    }

    pub fn set_position(&mut self, x: f32, y: f32, z: f32) -> &mut Self {
        self.position = Vec3::new(x, y, z);
        self
    }

    pub fn set_target(&mut self, x: f32, y: f32, z: f32) -> &mut Self {
        self.target = Vec3::new(x, y, z);
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
        self.view = look_at(&self.position, &self.target, &self.up);
        self.projection = project(self.fov, self.aspect, self.near, self.far);
        self
    }

    pub fn view_matrix(&self) -> &Mat4 {
        &self.view
    }

    pub fn projection_matrix(&self) -> &Mat4 {
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
        util::generate_buffers(indices, indexed_vertices, &mut vertices, &mut normals);
        Model {
            vertices,
            normals,
            instances: Vec::new(),
            instance_model_data: Vec::new(),
            instance_normals_data: Vec::new(),
        }
    }

    pub fn add_instance(&mut self, transform: Mat4) {
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
    pub model: Mat4,
    pub normals: Mat4,
}

impl Instance {
    pub fn new(model: Mat4) -> Self {
        Instance {
            model,
            normals: Mat4::IDENTITY,
        }
    }
}
