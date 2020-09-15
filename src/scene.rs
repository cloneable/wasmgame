extern crate js_sys;
extern crate std;
extern crate wasm_bindgen;
extern crate web_sys;

use std::option::{Option, Option::None, Option::Some};
use std::result::{Result, Result::Ok};

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

pub struct Model {
    pub buffer: web_sys::WebGlBuffer,
}

pub struct ModelBuilder<'a> {
    gl: &'a web_sys::WebGlRenderingContext,
    buffer: Option<web_sys::WebGlBuffer>,
}

impl<'a> ModelBuilder<'a> {
    #[must_use = "ModelBuilder must be finished."]
    pub fn new(gl: &'a web_sys::WebGlRenderingContext) -> Self {
        ModelBuilder { gl, buffer: None }
    }

    #[must_use = "ModelBuilder must be finished."]
    pub fn create_buffer(&mut self) -> Result<&mut Self, JsValue> {
        let buffer = self
            .gl
            .create_buffer()
            .ok_or_else(|| JsValue::from_str("create_buffer vbo_vertices error"))?;
        self.buffer = Some(buffer);
        Ok(self)
    }

    #[must_use = "ModelBuilder must be finished."]
    pub fn bind_buffer(&mut self) -> &mut Self {
        self.gl.bind_buffer(
            web_sys::WebGlRenderingContext::ARRAY_BUFFER,
            Some(self.buffer.as_ref().unwrap()),
        );
        self
    }

    #[must_use = "ModelBuilder must be finished."]
    pub fn set_buffer_data(&mut self, data: &[f32]) -> &mut Self {
        unsafe {
            let view = js_sys::Float32Array::view(data);
            self.gl.buffer_data_with_array_buffer_view(
                web_sys::WebGlRenderingContext::ARRAY_BUFFER,
                &view,
                web_sys::WebGlRenderingContext::STATIC_DRAW,
            );
        }
        self
    }

    #[must_use = "ModelBuilder must be finished."]
    pub fn set_vertex_attribute_pointer(&mut self, location: i32) -> &mut Self {
        self.gl.vertex_attrib_pointer_with_i32(
            location as u32,
            3,
            web_sys::WebGlRenderingContext::FLOAT,
            false,
            0,
            0,
        );
        self
    }

    pub fn build(&mut self) -> Model {
        Model {
            buffer: self.buffer.take().unwrap(),
        }
    }
}
