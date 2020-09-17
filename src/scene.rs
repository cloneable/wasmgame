extern crate js_sys;
extern crate log;
extern crate std;
extern crate wasm_bindgen;
extern crate web_sys;

use crate::game;
use crate::opengl;

use std::option::{Option, Option::None, Option::Some};
use std::result::{Result, Result::Ok};
use std::{vec, vec::Vec};

use crate::game::math;
use wasm_bindgen::JsValue;

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
            let mat_model_view = camera.view_matrix() * &instance.model;
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

pub struct BufferBuilder<'a> {
    ctx: &'a opengl::Context,

    buffer: Option<web_sys::WebGlBuffer>,
}

impl<'a> BufferBuilder<'a> {
    #[must_use = "BufferBuilder must be finished."]
    pub fn new(ctx: &'a opengl::Context) -> Self {
        BufferBuilder { ctx, buffer: None }
    }

    #[must_use = "BufferBuilder must be finished."]
    pub fn create_buffer(&mut self) -> Result<&mut Self, JsValue> {
        let buffer = self
            .ctx
            .gl
            .create_buffer()
            .ok_or_else(|| JsValue::from_str("create_buffer vbo_vertices error"))?;
        self.buffer = Some(buffer);
        Ok(self)
    }

    #[must_use = "BufferBuilder must be finished."]
    pub fn bind_buffer(&mut self) -> &mut Self {
        self.ctx.gl.bind_buffer(
            web_sys::WebGlRenderingContext::ARRAY_BUFFER,
            Some(self.buffer.as_ref().unwrap()),
        );
        self
    }

    #[must_use = "BufferBuilder must be finished."]
    pub fn set_buffer_data(&mut self, data: &[f32]) -> &mut Self {
        unsafe {
            let view = js_sys::Float32Array::view(data);
            self.ctx.gl.buffer_data_with_array_buffer_view(
                web_sys::WebGlRenderingContext::ARRAY_BUFFER,
                &view,
                web_sys::WebGlRenderingContext::STATIC_DRAW,
            );
        }
        self
    }

    #[must_use = "BufferBuilder must be finished."]
    pub fn set_vertex_attribute_pointer_vec3(&mut self, location: i32) -> &mut Self {
        self.ctx.gl.vertex_attrib_pointer_with_i32(
            location as u32,
            3,
            web_sys::WebGlRenderingContext::FLOAT,
            false,
            0,
            0,
        );
        self
    }

    #[must_use = "BufferBuilder must be finished."]
    pub fn set_vertex_attribute_pointer_mat4(&mut self, location: i32) -> &mut Self {
        for i in 0..=3 {
            self.ctx.gl.vertex_attrib_pointer_with_i32(
                (location + i) as u32,
                4,
                web_sys::WebGlRenderingContext::FLOAT,
                false,
                16 * 4,
                i * 4 * 4,
            );
        }
        self
    }

    #[must_use = "BufferBuilder must be finished."]
    pub fn set_vertex_attrib_divisor_mat4(&mut self, location: i32, divisor: usize) -> &mut Self {
        for i in 0..=3 {
            self.ctx
                .instanced_arrays_ext
                .vertex_attrib_divisor_angle(location as u32 + i, divisor as u32);
        }
        self
    }

    pub fn finish(&mut self) {
        self.buffer = None;
        self.ctx
            .gl
            .bind_buffer(web_sys::WebGlRenderingContext::ARRAY_BUFFER, None);
    }
}
