extern crate js_sys;
extern crate log;
extern crate std;
extern crate wasm_bindgen;
extern crate wasm_bindgen_futures;
extern crate web_sys;

pub mod math;

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
use std::{debug_assert_eq, panic};

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

pub fn generate_buffers(
    model_indices: &[u8],
    model_vertices: &[f32],
    vertices: &mut [f32],
    normals: &mut [f32],
) {
    debug_assert_eq!(model_indices.len() % 3, 0, "model_indices of wrong length");
    debug_assert_eq!(
        model_vertices.len() % 3,
        0,
        "model_vertices of wrong length"
    );
    debug_assert_eq!(
        vertices.len(),
        model_indices.len() * 3,
        "bad number of vertices"
    );
    debug_assert_eq!(
        normals.len(),
        model_indices.len() * 3,
        "bad number of normals"
    );
    let mut i = 0;
    while i < model_indices.len() {
        let ai = model_indices[i] as usize * 3;
        let a = math::Vec3::new(
            model_vertices[ai + 0],
            model_vertices[ai + 1],
            model_vertices[ai + 2],
        );
        let bi = model_indices[i + 1] as usize * 3;
        let b = math::Vec3::new(
            model_vertices[bi + 0],
            model_vertices[bi + 1],
            model_vertices[bi + 2],
        );
        let ci = model_indices[i + 2] as usize * 3;
        let c = math::Vec3::new(
            model_vertices[ci + 0],
            model_vertices[ci + 1],
            model_vertices[ci + 2],
        );

        let n = (&b - &a).cross(&(&c - &a)).normalize();

        let j = i * 3;
        vertices[j + 0] = a.x;
        vertices[j + 1] = a.y;
        vertices[j + 2] = a.z;
        vertices[j + 3] = b.x;
        vertices[j + 4] = b.y;
        vertices[j + 5] = b.z;
        vertices[j + 6] = c.x;
        vertices[j + 7] = c.y;
        vertices[j + 8] = c.z;

        normals[j + 0] = n.x;
        normals[j + 1] = n.y;
        normals[j + 2] = n.z;
        normals[j + 3] = n.x;
        normals[j + 4] = n.y;
        normals[j + 5] = n.z;
        normals[j + 6] = n.x;
        normals[j + 7] = n.y;
        normals[j + 8] = n.z;

        i += 3;
    }
}

#[cfg(test)]
pub mod tests {
    extern crate std;
    extern crate wasm_bindgen_test;
    use std::{assert_eq, panic};
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::*;

    #[wasm_bindgen_test]
    fn test_generate_buffers_simple() {
        // 2
        // | \
        // 0--1   4 above 0
        // | /
        // 3
        let model_vertices: [f32; 3 * 5] = [
            0.0, 0.0, 0.0, //br
            1.0, 0.0, 0.0, //br
            1.0, 0.0, -1.0, //br
            1.0, 0.0, 1.0, //br
            0.0, 1.0, 0.0, //br
        ];
        let model_indices: [u8; 3 * 3] = [0, 1, 2, 0, 1, 3, 0, 1, 4];
        let mut vertices: [f32; 3 * 9] = [0.0; 3 * 9];
        let mut normals: [f32; 3 * 9] = [0.0; 3 * 9];
        generate_buffers(&model_indices, &model_vertices, &mut vertices, &mut normals);

        let expect_vertices: [f32; 3 * 9] = [
            0.0, 0.0, 0.0, //br
            1.0, 0.0, 0.0, //br
            1.0, 0.0, -1.0, //br
            0.0, 0.0, 0.0, //br
            1.0, 0.0, 0.0, //br
            1.0, 0.0, 1.0, //br
            0.0, 0.0, 0.0, //br
            1.0, 0.0, 0.0, //br
            0.0, 1.0, 0.0, //br
        ];
        let expect_normals: [f32; 3 * 9] = [
            0.0, 1.0, 0.0, //br
            0.0, 1.0, 0.0, //br
            0.0, 1.0, 0.0, //br
            0.0, -1.0, 0.0, //br
            0.0, -1.0, 0.0, //br
            0.0, -1.0, 0.0, //br
            0.0, 0.0, 1.0, //br
            0.0, 0.0, 1.0, //br
            0.0, 0.0, 1.0, //br
        ];
        assert_eq!(vertices, expect_vertices);
        assert_eq!(normals, expect_normals);
    }
}
