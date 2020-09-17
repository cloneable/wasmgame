extern crate js_sys;
extern crate log;
extern crate std;
extern crate wasm_bindgen;
extern crate wasm_bindgen_futures;
extern crate web_sys;

use std::convert::From;
use std::convert::Into;
use std::option::{Option::None, Option::Some};
use std::result::{Result, Result::Err, Result::Ok};
use std::string::String;

use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;

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
}

pub struct VertexArrayObject<'a> {
    ctx: &'a Context,
    vao: web_sys::WebGlVertexArrayObject,
}

impl<'a> VertexArrayObject<'a> {
    pub fn create(ctx: &'a Context) -> Result<Self, JsValue> {
        let vao = ctx
            .vertex_array_object_ext
            .create_vertex_array_oes()
            .ok_or_else(|| JsValue::from_str("create_vertex_array_oes vao error"))?;
        Ok(VertexArrayObject { ctx, vao })
    }

    pub fn bind(&mut self) -> &mut Self {
        // TODO: track binding and debug_assert
        self.ctx
            .vertex_array_object_ext
            .bind_vertex_array_oes(Some(&self.vao));
        self
    }

    pub fn unbind(&mut self) -> &mut Self {
        self.ctx.vertex_array_object_ext.bind_vertex_array_oes(None);
        self
    }
}

impl<'a> std::ops::Drop for VertexArrayObject<'a> {
    fn drop(&mut self) {
        // TODO: enable delete once app structure is sorted out.
        log::debug!("deleting vao");
        self.ctx
            .vertex_array_object_ext
            .delete_vertex_array_oes(Some(&self.vao));
    }
}

pub struct ArrayBuffer<'a> {
    ctx: &'a Context,
    buffer: web_sys::WebGlBuffer,
}

impl<'a> std::ops::Drop for ArrayBuffer<'a> {
    fn drop(&mut self) {
        // TODO: enable delete once app structure is sorted out.
        //log::debug!("deleting buffer");
        //self.ctx.gl.delete_buffer(Some(&self.buffer));
    }
}

impl<'a> ArrayBuffer<'a> {
    pub fn create(ctx: &'a Context) -> Result<Self, JsValue> {
        let buffer = ctx
            .gl
            .create_buffer()
            .ok_or_else(|| JsValue::from_str("create_buffer vbo_vertices error"))?;
        Ok(ArrayBuffer { ctx, buffer })
    }

    pub fn bind(&mut self) -> &mut Self {
        // TODO: track binding and debug_assert
        self.ctx.gl.bind_buffer(
            web_sys::WebGlRenderingContext::ARRAY_BUFFER,
            Some(&self.buffer),
        );
        self
    }

    pub fn unbind(&mut self) {
        self.ctx
            .gl
            .bind_buffer(web_sys::WebGlRenderingContext::ARRAY_BUFFER, None);
    }

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

    pub fn set_vertex_attrib_divisor_mat4(&mut self, location: i32, divisor: usize) -> &mut Self {
        for i in 0..=3 {
            self.ctx
                .instanced_arrays_ext
                .vertex_attrib_divisor_angle(location as u32 + i, divisor as u32);
        }
        self
    }
}
