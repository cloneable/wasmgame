extern crate js_sys;
extern crate log;
extern crate std;
extern crate wasm_bindgen;
extern crate wasm_bindgen_futures;
extern crate web_sys;

use std::boxed::Box;
use std::cell::RefCell;
use std::clone::Clone;
use std::convert::AsRef;
use std::convert::From;
use std::convert::Into;
use std::ops::FnMut;
use std::option::{Option, Option::None, Option::Some};
use std::rc::Rc;
use std::result::{Result, Result::Err, Result::Ok};
use std::string::String;
use std::{assert_eq, panic};

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;

pub trait Renderer {
    fn setup(&mut self, ctx: &RenderingContext) -> Result<(), JsValue>;
    fn render(&mut self, ctx: &RenderingContext, millis: f64) -> Result<(), JsValue>;
    fn done(&self) -> bool;
}

pub struct RenderingContext {
    /// https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.WebGlRenderingContext.html
    pub gl: web_sys::WebGlRenderingContext,
    /// https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.OesVertexArrayObject.html
    pub vertex_array_object_ext: web_sys::OesVertexArrayObject,
    /// https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.AngleInstancedArrays.html
    pub instanced_arrays_ext: web_sys::AngleInstancedArrays,
}

impl RenderingContext {
    pub fn create_vertex_shader(&self, glsl: &str) -> Result<web_sys::WebGlShader, JsValue> {
        self.create_shader(glsl, web_sys::WebGlRenderingContext::VERTEX_SHADER)
    }

    pub fn create_fragment_shader(&self, glsl: &str) -> Result<web_sys::WebGlShader, JsValue> {
        self.create_shader(glsl, web_sys::WebGlRenderingContext::FRAGMENT_SHADER)
    }

    fn create_shader(&self, glsl: &str, type_: u32) -> Result<web_sys::WebGlShader, JsValue> {
        let shader = self
            .gl
            .create_shader(type_)
            .ok_or_else(|| JsValue::from_str("create_shader failed"))?;
        self.gl.shader_source(&shader, glsl);
        self.gl.compile_shader(&shader);

        if self
            .gl
            .get_shader_parameter(&shader, web_sys::WebGlRenderingContext::COMPILE_STATUS)
            .as_bool()
            .unwrap_or(false)
        {
            Ok(shader)
        } else {
            let info = self
                .gl
                .get_shader_info_log(&shader)
                .unwrap_or_else(|| String::from("unknown error"));
            log::error!("shader error: {}", info);
            Err(info.into())
        }
    }

    pub fn link_program(
        &self,
        vertex_shader: &web_sys::WebGlShader,
        fragment_shader: &web_sys::WebGlShader,
    ) -> Result<web_sys::WebGlProgram, JsValue> {
        let program = self
            .gl
            .create_program()
            .ok_or_else(|| JsValue::from_str("Unable to create program object"))?;

        self.gl.attach_shader(&program, vertex_shader);
        self.gl.attach_shader(&program, fragment_shader);
        self.gl.link_program(&program);

        if self
            .gl
            .get_program_parameter(&program, web_sys::WebGlRenderingContext::LINK_STATUS)
            .as_bool()
            .unwrap_or(false)
        {
            Ok(program)
        } else {
            let info = self
                .gl
                .get_program_info_log(&program)
                .unwrap_or_else(|| String::from("unknown error"));
            log::error!("program error: {}", info);
            Err(info.into())
        }
    }
}

type RequestAnimationFrameCallback = Closure<dyn FnMut(f64) + 'static>;

fn request_animation_frame_helper(callback: Option<&RequestAnimationFrameCallback>) {
    web_sys::window()
        .unwrap()
        .request_animation_frame(callback.unwrap().as_ref().unchecked_ref())
        .unwrap();
}

pub struct Engine {
    pub ctx: RenderingContext,
    renderer: Rc<RefCell<dyn Renderer>>,
}

impl Engine {
    pub fn new(
        gl: web_sys::WebGlRenderingContext,
        renderer: Rc<RefCell<dyn Renderer>>,
    ) -> Rc<Self> {
        let vertex_array_object_ext = gl
            .get_extension("OES_vertex_array_object")
            .unwrap()
            .unwrap()
            .unchecked_into::<web_sys::OesVertexArrayObject>();
        let instanced_arrays_ext = gl
            .get_extension("ANGLE_instanced_arrays")
            .unwrap()
            .unwrap()
            .unchecked_into::<web_sys::AngleInstancedArrays>();
        Rc::new(Self {
            ctx: RenderingContext {
                gl,
                vertex_array_object_ext,
                instanced_arrays_ext,
            },
            renderer,
        })
    }

    pub fn start(self: Rc<Self>) -> Result<(), JsValue> {
        self.renderer.borrow_mut().setup(&self.ctx)?;
        // Part of this is taken from the wasm-bindgen guide.
        // This kinda works for now, but needs to be checked for
        // leaks.
        // TODO: Check if renderer, callback instances not freed.
        // TODO: See if there's a better/cleaner way to do this.
        let callback = Rc::new(RefCell::new(None as Option<RequestAnimationFrameCallback>));
        let c = callback.clone();
        *callback.borrow_mut() = Some(Closure::wrap(Box::new(move |millis: f64| {
            if self.renderer.borrow().done() {
                let _ = c.borrow_mut().take();
                log::info!("wasmgame ending");
                return;
            }

            self.renderer
                .borrow_mut()
                .render(&self.ctx, millis)
                .unwrap();

            let c0 = c.clone();
            request_animation_frame_helper(c0.borrow().as_ref());
        }) as Box<dyn FnMut(f64) + 'static>));

        request_animation_frame_helper(callback.borrow().as_ref());
        Ok(())
    }
}

// TODO: move into math module

#[derive(Copy, Clone)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
    pub fn is_zero(&self) -> bool {
        self.x == 0.0 && self.y == 0.0 && self.z == 0.0
    }
}

impl std::ops::Sub for Vec3 {
    type Output = Vec3;
    fn sub(self, other: Vec3) -> Vec3 {
        Vec3 {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

pub fn cross(a: Vec3, b: Vec3) -> Vec3 {
    Vec3 {
        x: a.y * b.z - a.z * b.y,
        y: a.z * b.x - a.x * b.z,
        z: a.x * b.y - a.y * b.x,
    }
}

pub fn normalize(v: Vec3) -> Vec3 {
    if v.is_zero() {
        return v;
    }
    let length = (v.x * v.x + v.y * v.y + v.z * v.z).sqrt();
    Vec3 {
        x: v.x / length,
        y: v.y / length,
        z: v.z / length,
    }
}

pub fn interleave_with_normals(indices: &[u8], vertices: &[f32], out: &mut [f32]) {
    // assert_eq!(indices.len() % 3, 0, "indices of wrong length");
    // assert_eq!(indices.len(), out.len() * 6, "bad size");
    let mut idx = 0;
    while idx < indices.len() {
        let ui = indices[idx] as usize * 3;
        let u = Vec3::new(vertices[ui + 0], vertices[ui + 1], vertices[ui + 2]);
        let vi = indices[idx + 1] as usize * 3;
        let v = Vec3::new(vertices[vi + 0], vertices[vi + 1], vertices[vi + 2]);
        let wi = indices[idx + 2] as usize * 3;
        let w = Vec3::new(vertices[wi + 0], vertices[wi + 1], vertices[wi + 2]);

        let n = normalize(cross(v - u, w - u));

        out[idx * 6 + 0] = u.x;
        out[idx * 6 + 1] = u.y;
        out[idx * 6 + 2] = u.z;

        out[idx * 6 + 3] = n.x;
        out[idx * 6 + 4] = n.y;
        out[idx * 6 + 5] = n.z;

        out[idx * 6 + 6] = v.x;
        out[idx * 6 + 7] = v.y;
        out[idx * 6 + 8] = v.z;

        out[idx * 6 + 9] = n.x;
        out[idx * 6 + 10] = n.y;
        out[idx * 6 + 11] = n.z;

        out[idx * 6 + 12] = w.x;
        out[idx * 6 + 13] = w.y;
        out[idx * 6 + 14] = w.z;

        out[idx * 6 + 15] = n.x;
        out[idx * 6 + 16] = n.y;
        out[idx * 6 + 17] = n.z;

        idx += 3;
    }
    // assert_eq!(idx, indices.len(), "bad idx");
}
