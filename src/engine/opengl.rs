extern crate js_sys;
extern crate log;
extern crate std;
extern crate wasm_bindgen;
extern crate wasm_bindgen_futures;
extern crate web_sys;

use std::cell::RefCell;
use std::convert::From;
use std::convert::Into;
use std::option::{Option::None, Option::Some};
use std::result::{Result, Result::Err, Result::Ok};
use std::string::String;
use std::{assert_eq, assert_ne, panic};

use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;

pub struct Context {
    /// https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.WebGlRenderingContext.html
    pub gl: web_sys::WebGlRenderingContext,
    /// https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.OesVertexArrayObject.html
    pub vertex_array_object_ext: web_sys::OesVertexArrayObject,
    /// https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.AngleInstancedArrays.html
    pub instanced_arrays_ext: web_sys::AngleInstancedArrays,

    next_object_id: RefCell<u32>,
    bound_array_buffer: RefCell<u32>,
    bound_framebuffer: RefCell<u32>,
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
            next_object_id: RefCell::new(1),
            bound_array_buffer: RefCell::new(0),
            bound_framebuffer: RefCell::new(0),
        })
    }

    pub fn next_object_id(&self) -> u32 {
        let id = *self.next_object_id.borrow();
        *self.next_object_id.borrow_mut() = id + 1;
        id
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
        log::debug!("deleting vao");
        self.ctx
            .vertex_array_object_ext
            .delete_vertex_array_oes(Some(&self.vao));
    }
}

pub struct ArrayBuffer<'a> {
    ctx: &'a Context,
    id: u32,
    buffer: web_sys::WebGlBuffer,
}

impl<'a> std::ops::Drop for ArrayBuffer<'a> {
    fn drop(&mut self) {
        self.assert_unbound();
        log::debug!("deleting buffer (not really)");
        // TODO: enable delete once app structure is sorted out.
        //self.ctx.gl.delete_buffer(Some(&self.buffer));
    }
}

impl<'a> ArrayBuffer<'a> {
    pub fn create(ctx: &'a Context) -> Result<Self, JsValue> {
        let buffer = ctx
            .gl
            .create_buffer()
            .ok_or_else(|| JsValue::from_str("create_buffer vbo_vertices error"))?;
        let id = ctx.next_object_id();
        Ok(ArrayBuffer { ctx, id, buffer })
    }

    pub fn bind(&mut self) -> &mut Self {
        self.assert_unbound_and_bind();
        self.ctx.gl.bind_buffer(
            web_sys::WebGlRenderingContext::ARRAY_BUFFER,
            Some(&self.buffer),
        );
        self
    }

    pub fn unbind(&mut self) {
        self.assert_bound_and_unbind();
        self.ctx
            .gl
            .bind_buffer(web_sys::WebGlRenderingContext::ARRAY_BUFFER, None);
    }

    pub fn set_buffer_data(&mut self, data: &[f32]) -> &mut Self {
        self.assert_bound();
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

    pub fn set_vertex_attribute_pointer_vec3(&mut self, attribute: &Attribute) -> &mut Self {
        self.assert_bound();
        self.ctx.gl.vertex_attrib_pointer_with_i32(
            attribute.location,
            3,
            web_sys::WebGlRenderingContext::FLOAT,
            false,
            0,
            0,
        );
        self
    }

    pub fn set_vertex_attribute_pointer_mat4(&mut self, attribute: &Attribute) -> &mut Self {
        self.assert_bound();
        for i in 0..=3 {
            self.ctx.gl.vertex_attrib_pointer_with_i32(
                attribute.location + i,
                4,
                web_sys::WebGlRenderingContext::FLOAT,
                false,
                16 * 4,
                i as i32 * 4 * 4,
            );
        }
        self
    }

    pub fn set_vertex_attrib_divisor_mat4(
        &mut self,
        attribute: &Attribute,
        divisor: usize,
    ) -> &mut Self {
        self.assert_bound();
        for i in 0..=3 {
            self.ctx
                .instanced_arrays_ext
                .vertex_attrib_divisor_angle(attribute.location + i, divisor as u32);
        }
        self
    }

    fn assert_bound(&self) {
        assert_eq!(*self.ctx.bound_array_buffer.borrow(), self.id);
    }

    fn assert_bound_and_unbind(&mut self) {
        assert_eq!(*self.ctx.bound_array_buffer.borrow(), self.id);
        *self.ctx.bound_array_buffer.borrow_mut() = 0;
    }

    fn assert_unbound(&self) {
        assert_ne!(*self.ctx.bound_array_buffer.borrow(), self.id);
    }

    fn assert_unbound_and_bind(&mut self) {
        assert_ne!(*self.ctx.bound_array_buffer.borrow(), self.id);
        *self.ctx.bound_array_buffer.borrow_mut() = self.id;
    }
}

pub struct Uniform<'a> {
    ctx: &'a Context,
    location: web_sys::WebGlUniformLocation,
}

impl<'a> Uniform<'a> {
    pub fn find<'b>(ctx: &'a Context, program: &'b Program, name: &str) -> Result<Self, JsValue> {
        let location = ctx
            .gl
            .get_uniform_location(&program.program, name)
            .ok_or_else(|| JsValue::from_str("get_uniform_location error: view"))?;
        Ok(Uniform { ctx, location })
    }

    pub fn set_mat4(&mut self, data: &[f32]) {
        self.ctx
            .gl
            .uniform_matrix4fv_with_f32_array(Some(&self.location), false, data);
    }
}

pub struct Attribute<'a> {
    ctx: &'a Context,
    location: u32,
    // TODO: use generic type instead of slots.
    slots: usize,
}

impl<'a> Attribute<'a> {
    #[allow(dead_code)]
    pub fn find<'b>(
        ctx: &'a Context,
        program: &'b Program,
        name: &str,
        slots: usize,
    ) -> Result<Self, JsValue> {
        let location = ctx.gl.get_attrib_location(&program.program, name);
        if location == -1 {
            // TODO: add attribute name
            return Err(JsValue::from_str("attribute not found"));
        }
        Ok(Attribute {
            ctx,
            location: location as u32,
            slots,
        })
    }

    pub fn bind<'b>(
        ctx: &'a Context,
        program: &'b Program,
        location: u32,
        name: &str,
        slots: usize,
    ) -> Result<Self, JsValue> {
        ctx.gl
            .bind_attrib_location(&program.program, location, name);
        Ok(Attribute {
            ctx,
            location,
            slots,
        })
    }

    pub fn enable(&mut self) {
        for i in 0..self.slots {
            self.ctx
                .gl
                .enable_vertex_attrib_array(self.location + i as u32);
        }
    }
}

pub struct Program<'a> {
    ctx: &'a Context,
    pub program: web_sys::WebGlProgram,
}

impl<'a> Program<'a> {
    pub fn create(ctx: &'a Context) -> Result<Self, JsValue> {
        let program = ctx
            .gl
            .create_program()
            .ok_or_else(|| JsValue::from_str("Unable to create program object"))?;
        Ok(Program { ctx, program })
    }

    pub fn attach_shader(&mut self, shader: &Shader) {
        self.ctx.gl.attach_shader(&self.program, &shader.shader);
    }

    pub fn link(&mut self) -> Result<(), JsValue> {
        self.ctx.gl.link_program(&self.program);
        if self
            .ctx
            .gl
            .get_program_parameter(&self.program, web_sys::WebGlRenderingContext::LINK_STATUS)
            .as_bool()
            .unwrap_or(false)
        {
            Ok(())
        } else {
            let info = self
                .ctx
                .gl
                .get_program_info_log(&self.program)
                .unwrap_or_else(|| String::from("unknown error"));
            log::error!("program error: {}", info);
            Err(info.into())
        }
    }

    pub fn r#use(&mut self) {
        self.ctx.gl.use_program(Some(&self.program));
    }
}

impl<'a> std::ops::Drop for Program<'a> {
    fn drop(&mut self) {
        log::debug!("deleting program");
        self.ctx.gl.delete_program(Some(&self.program));
    }
}

pub struct Shader<'a> {
    ctx: &'a Context,
    shader: web_sys::WebGlShader,
}

pub enum ShaderType {
    Vertex,
    Fragment,
}

impl<'a> Shader<'a> {
    pub fn create(ctx: &'a Context, type_: ShaderType) -> Result<Self, JsValue> {
        let pt = match type_ {
            ShaderType::Vertex => web_sys::WebGlRenderingContext::VERTEX_SHADER,
            ShaderType::Fragment => web_sys::WebGlRenderingContext::FRAGMENT_SHADER,
        };
        let shader = ctx
            .gl
            .create_shader(pt)
            .ok_or_else(|| JsValue::from_str("create_shader failed"))?;
        Ok(Shader { ctx, shader })
    }

    pub fn compile_source(&mut self, glsl: &str) -> Result<(), JsValue> {
        self.ctx.gl.shader_source(&self.shader, glsl);
        self.ctx.gl.compile_shader(&self.shader);
        if self
            .ctx
            .gl
            .get_shader_parameter(&self.shader, web_sys::WebGlRenderingContext::COMPILE_STATUS)
            .as_bool()
            .unwrap_or(false)
        {
            Ok(())
        } else {
            let info = self
                .ctx
                .gl
                .get_shader_info_log(&self.shader)
                .unwrap_or_else(|| String::from("unknown error"));
            log::error!("shader error: {}", info);
            Err(info.into())
        }
    }
}

impl<'a> std::ops::Drop for Shader<'a> {
    fn drop(&mut self) {
        log::debug!("deleting shader");
        self.ctx.gl.delete_shader(Some(&self.shader));
    }
}

pub struct Framebuffer<'a> {
    ctx: &'a Context,
    id: u32,
    buffer: web_sys::WebGlFramebuffer,
}

impl<'a> Framebuffer<'a> {
    pub fn create(ctx: &'a Context) -> Result<Self, JsValue> {
        let buffer = ctx
            .gl
            .create_framebuffer()
            .ok_or_else(|| JsValue::from_str("create_framebuffer error"))?;
        let id = ctx.next_object_id();
        Ok(Framebuffer { ctx, id, buffer })
    }

    pub fn bind(&mut self) {
        self.assert_unbound_and_bind();
        self.ctx.gl.bind_framebuffer(
            web_sys::WebGlRenderingContext::FRAMEBUFFER,
            Some(&self.buffer),
        )
    }

    pub fn unbind(&mut self) {
        self.assert_bound_and_unbind();
        self.ctx
            .gl
            .bind_framebuffer(web_sys::WebGlRenderingContext::FRAMEBUFFER, None)
    }

    pub fn check_status(&self) -> u32 {
        self.assert_bound();
        self.ctx
            .gl
            .check_framebuffer_status(web_sys::WebGlRenderingContext::FRAMEBUFFER)
    }

    pub fn read_pixels(
        &self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        format: u32,
        type_: u32,
        pixels: &mut [u8],
    ) -> Result<(), JsValue> {
        self.assert_bound();
        self.ctx
            .gl
            .read_pixels_with_opt_u8_array(x, y, width, height, format, type_, Some(pixels))
    }

    fn assert_bound(&self) {
        assert_eq!(*self.ctx.bound_framebuffer.borrow(), self.id);
    }

    fn assert_bound_and_unbind(&mut self) {
        assert_eq!(*self.ctx.bound_framebuffer.borrow(), self.id);
        *self.ctx.bound_framebuffer.borrow_mut() = 0;
    }

    fn assert_unbound(&self) {
        assert_ne!(*self.ctx.bound_framebuffer.borrow(), self.id);
    }

    fn assert_unbound_and_bind(&mut self) {
        assert_ne!(*self.ctx.bound_framebuffer.borrow(), self.id);
        *self.ctx.bound_framebuffer.borrow_mut() = self.id;
    }
}

impl<'a> std::ops::Drop for Framebuffer<'a> {
    fn drop(&mut self) {
        self.assert_unbound();
        log::debug!("deleting framebuffer");
        self.ctx.gl.delete_framebuffer(Some(&self.buffer));
    }
}
