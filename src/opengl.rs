extern crate js_sys;
extern crate log;
extern crate std;
extern crate wasm_bindgen;
extern crate wasm_bindgen_futures;
extern crate web_sys;

use std::convert::From;
use std::convert::Into;
use std::result::{Result, Result::Err, Result::Ok};
use std::string::String;

use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;

use crate::scene;

pub struct Context {
    /// https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.WebGlRenderingContext.html
    pub gl: web_sys::WebGlRenderingContext,
    /// https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.OesVertexArrayObject.html
    pub vertex_array_object_ext: web_sys::OesVertexArrayObject,
    /// https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.AngleInstancedArrays.html
    pub instanced_arrays_ext: web_sys::AngleInstancedArrays,
}

impl Context {
    pub fn from_canvas(canvas: &web_sys::HtmlCanvasElement) -> Result<Self, JsValue> {
        let gl = canvas
            .get_context("webgl")
            .expect("getContext failed")
            .expect("unsupported context type")
            .dyn_into::<web_sys::WebGlRenderingContext>()
            .expect("context of unexpected type");
        let vertex_array_object_ext = gl
            .get_extension("OES_vertex_array_object")
            .unwrap()
            .unwrap()
            .unchecked_into::<web_sys::OesVertexArrayObject>();
        // TODO: try ANGLEInstancedArrays if ANGLE_instanced_arrays doesn't work.
        let instanced_arrays_ext = gl
            .get_extension("ANGLE_instanced_arrays")
            .unwrap()
            .unwrap()
            .unchecked_into::<web_sys::AngleInstancedArrays>();
        // TODO: find better place for this. some init func?
        gl.enable(web_sys::WebGlRenderingContext::CULL_FACE);
        gl.enable(web_sys::WebGlRenderingContext::DEPTH_TEST);
        gl.hint(
            web_sys::WebGlRenderingContext::GENERATE_MIPMAP_HINT,
            web_sys::WebGlRenderingContext::NICEST,
        );
        Ok(Context {
            gl,
            vertex_array_object_ext,
            instanced_arrays_ext,
        })
    }

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

    #[must_use = "BufferBuilder must be finished."]
    pub fn buffer_builder(&self) -> scene::BufferBuilder {
        scene::BufferBuilder::new(&self)
    }
}
