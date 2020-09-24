extern crate js_sys;
extern crate log;
extern crate std;
extern crate wasm_bindgen;
extern crate wasm_bindgen_futures;
extern crate web_sys;

use std::cell::RefCell;
use std::clone::Clone;
use std::convert::From;
use std::convert::Into;
use std::option::{Option, Option::None, Option::Some};
use std::rc::Rc;
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
    bound_vertex_array_buffer: RefCell<u32>,
    bound_array_buffer: RefCell<u32>,
    bound_framebuffer: RefCell<u32>,
    bound_renderbuffer: RefCell<u32>,
    bound_texture: RefCell<u32>,
}

impl Context {
    pub fn from_canvas(canvas: &web_sys::HtmlCanvasElement) -> Result<Self, JsValue> {
        let gl = canvas
            .get_context_with_context_options("webgl", &web_sys::WebGlContextAttributes::new())
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
            bound_vertex_array_buffer: RefCell::new(0),
            bound_array_buffer: RefCell::new(0),
            bound_framebuffer: RefCell::new(0),
            bound_renderbuffer: RefCell::new(0),
            bound_texture: RefCell::new(0),
        })
    }

    pub fn next_object_id(&self) -> u32 {
        let id = *self.next_object_id.borrow();
        *self.next_object_id.borrow_mut() = id + 1;
        id
    }
}

pub struct VertexArrayObject {
    ctx: Rc<Context>,
    id: u32,
    vao: web_sys::WebGlVertexArrayObject,
}

impl VertexArrayObject {
    pub fn create(ctx: &Rc<Context>) -> Result<Self, JsValue> {
        let vao = ctx
            .vertex_array_object_ext
            .create_vertex_array_oes()
            .ok_or_else(|| JsValue::from_str("create_vertex_array_oes vao error"))?;
        let id = ctx.next_object_id();
        Ok(VertexArrayObject {
            ctx: ctx.clone(),
            id,
            vao,
        })
    }

    pub fn bind(&mut self) -> &mut Self {
        self.assert_unbound_and_bind();
        self.ctx
            .vertex_array_object_ext
            .bind_vertex_array_oes(Some(&self.vao));
        self
    }

    pub fn unbind(&mut self) -> &mut Self {
        self.assert_bound_and_unbind();
        self.ctx.vertex_array_object_ext.bind_vertex_array_oes(None);
        self
    }

    fn assert_bound(&self) {
        assert_eq!(*self.ctx.bound_vertex_array_buffer.borrow(), self.id);
    }

    fn assert_bound_and_unbind(&mut self) {
        assert_eq!(*self.ctx.bound_vertex_array_buffer.borrow(), self.id);
        *self.ctx.bound_vertex_array_buffer.borrow_mut() = 0;
    }

    fn assert_unbound(&self) {
        assert_ne!(*self.ctx.bound_vertex_array_buffer.borrow(), self.id);
    }

    fn assert_unbound_and_bind(&mut self) {
        assert_ne!(*self.ctx.bound_vertex_array_buffer.borrow(), self.id);
        *self.ctx.bound_vertex_array_buffer.borrow_mut() = self.id;
    }
}

impl std::ops::Drop for VertexArrayObject {
    fn drop(&mut self) {
        log::debug!("deleting vao");
        self.ctx
            .vertex_array_object_ext
            .delete_vertex_array_oes(Some(&self.vao));
    }
}

pub struct ArrayBuffer {
    ctx: Rc<Context>,
    id: u32,
    buffer: web_sys::WebGlBuffer,
}

impl ArrayBuffer {
    pub fn create(ctx: &Rc<Context>) -> Result<Self, JsValue> {
        let buffer = ctx
            .gl
            .create_buffer()
            .ok_or_else(|| JsValue::from_str("create_buffer vbo_vertices error"))?;
        let id = ctx.next_object_id();
        Ok(ArrayBuffer {
            ctx: ctx.clone(),
            id,
            buffer,
        })
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

    pub fn allocate_dynamic(&mut self, size: usize) -> &mut Self {
        self.assert_bound();
        self.ctx.gl.buffer_data_with_i32(
            web_sys::WebGlRenderingContext::ARRAY_BUFFER,
            size as i32,
            web_sys::WebGlRenderingContext::DYNAMIC_DRAW,
        );
        self
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

    pub fn set_buffer_sub_data(&mut self, offset: i32, data: &[f32]) -> &mut Self {
        self.assert_bound();
        unsafe {
            let view = js_sys::Float32Array::view(data);
            self.ctx.gl.buffer_sub_data_with_i32_and_array_buffer_view(
                web_sys::WebGlRenderingContext::ARRAY_BUFFER,
                offset,
                &view,
            );
        }
        self
    }

    pub fn set_vertex_attribute_pointer_vec3(&mut self, attribute: Attribute) -> &mut Self {
        self.assert_bound();
        self.ctx.gl.vertex_attrib_pointer_with_i32(
            attribute.0,
            3,
            web_sys::WebGlRenderingContext::FLOAT,
            false,
            3 * 4,
            0,
        );
        self
    }

    pub fn set_vertex_attribute_pointer_mat4(&mut self, attribute: Attribute) -> &mut Self {
        self.assert_bound();
        for i in 0..attribute.1 {
            self.ctx.gl.vertex_attrib_pointer_with_i32(
                attribute.0 + i as u32,
                4,
                web_sys::WebGlRenderingContext::FLOAT,
                false,
                16 * 4,
                i as i32 * 4 * 4,
            );
        }
        self
    }

    pub fn set_vertex_attrib_divisor(&mut self, attribute: Attribute, divisor: usize) -> &mut Self {
        self.assert_bound();
        for i in 0..attribute.1 {
            self.ctx
                .instanced_arrays_ext
                .vertex_attrib_divisor_angle(attribute.0 + i as u32, divisor as u32);
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

impl std::ops::Drop for ArrayBuffer {
    fn drop(&mut self) {
        self.assert_unbound();
        log::debug!("deleting buffer");
        self.ctx.gl.delete_buffer(Some(&self.buffer));
    }
}

pub struct Uniform {
    ctx: Rc<Context>,
    location: web_sys::WebGlUniformLocation,
}

impl Uniform {
    pub fn find<'b>(ctx: &Rc<Context>, program: &'b Program, name: &str) -> Result<Self, JsValue> {
        let location = ctx
            .gl
            .get_uniform_location(&program.program, name)
            .ok_or_else(|| JsValue::from_str("get_uniform_location error: view"))?;
        Ok(Uniform {
            ctx: ctx.clone(),
            location,
        })
    }

    pub fn set_mat4(&mut self, data: &[f32]) {
        self.ctx
            .gl
            .uniform_matrix4fv_with_f32_array(Some(&self.location), false, data);
    }
}

#[derive(Copy, Clone)]
pub struct Attribute(pub u32, pub usize);

impl Attribute {
    pub fn bind(&self, ctx: &Rc<Context>, program: &Program, name: &str) {
        ctx.gl.bind_attrib_location(&program.program, self.0, name)
    }

    pub fn enable(&self, ctx: &Rc<Context>) {
        for i in 0..self.1 {
            ctx.gl.enable_vertex_attrib_array(self.0 + i as u32);
        }
    }

    pub fn disable(&self, ctx: &Rc<Context>) {
        for i in 0..self.1 {
            ctx.gl.disable_vertex_attrib_array(self.0 + i as u32);
        }
    }
}

pub struct Program {
    ctx: Rc<Context>,
    pub program: web_sys::WebGlProgram,
}

impl Program {
    pub fn create(ctx: &Rc<Context>) -> Result<Self, JsValue> {
        let program = ctx
            .gl
            .create_program()
            .ok_or_else(|| JsValue::from_str("Unable to create program object"))?;
        Ok(Program {
            ctx: ctx.clone(),
            program,
        })
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

impl<'a> std::ops::Drop for Program {
    fn drop(&mut self) {
        log::debug!("deleting program");
        self.ctx.gl.delete_program(Some(&self.program));
    }
}

pub struct Shader {
    ctx: Rc<Context>,
    shader: web_sys::WebGlShader,
}

pub enum ShaderType {
    Vertex,
    Fragment,
}

impl Shader {
    pub fn create(ctx: &Rc<Context>, type_: ShaderType) -> Result<Self, JsValue> {
        let pt = match type_ {
            ShaderType::Vertex => web_sys::WebGlRenderingContext::VERTEX_SHADER,
            ShaderType::Fragment => web_sys::WebGlRenderingContext::FRAGMENT_SHADER,
        };
        let shader = ctx
            .gl
            .create_shader(pt)
            .ok_or_else(|| JsValue::from_str("create_shader failed"))?;
        Ok(Shader {
            ctx: ctx.clone(),
            shader,
        })
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

impl std::ops::Drop for Shader {
    fn drop(&mut self) {
        log::debug!("deleting shader");
        self.ctx.gl.delete_shader(Some(&self.shader));
    }
}

pub struct Framebuffer {
    ctx: Rc<Context>,
    id: u32,
    buffer: web_sys::WebGlFramebuffer,
}

impl Framebuffer {
    pub fn create(ctx: &Rc<Context>) -> Result<Self, JsValue> {
        let buffer = ctx
            .gl
            .create_framebuffer()
            .ok_or_else(|| JsValue::from_str("create_framebuffer error"))?;
        let id = ctx.next_object_id();
        Ok(Framebuffer {
            ctx: ctx.clone(),
            id,
            buffer,
        })
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

    pub fn texture2d_as_colorbuffer(&mut self, texture: &Texture2D) {
        self.assert_bound();
        self.ctx.gl.framebuffer_texture_2d(
            web_sys::WebGlRenderingContext::FRAMEBUFFER,
            web_sys::WebGlRenderingContext::COLOR_ATTACHMENT0,
            web_sys::WebGlRenderingContext::TEXTURE_2D,
            Some(&texture.texture),
            0,
        )
    }

    pub fn renderbuffer_as_depthbuffer(&mut self, buffer: &Renderbuffer) {
        self.assert_bound();
        self.ctx.gl.framebuffer_renderbuffer(
            web_sys::WebGlRenderingContext::FRAMEBUFFER,
            web_sys::WebGlRenderingContext::DEPTH_ATTACHMENT,
            web_sys::WebGlRenderingContext::RENDERBUFFER,
            Some(&buffer.buffer),
        )
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

impl std::ops::Drop for Framebuffer {
    fn drop(&mut self) {
        self.assert_unbound();
        log::debug!("deleting framebuffer");
        self.ctx.gl.delete_framebuffer(Some(&self.buffer));
    }
}

pub struct Renderbuffer {
    ctx: Rc<Context>,
    id: u32,
    buffer: web_sys::WebGlRenderbuffer,
}

impl Renderbuffer {
    pub fn create(ctx: &Rc<Context>) -> Result<Self, JsValue> {
        let buffer = ctx
            .gl
            .create_renderbuffer()
            .ok_or_else(|| JsValue::from_str("create_renderbuffer error"))?;
        let id = ctx.next_object_id();
        Ok(Renderbuffer {
            ctx: ctx.clone(),
            id,
            buffer,
        })
    }

    pub fn bind(&mut self) {
        self.assert_unbound_and_bind();
        self.ctx.gl.bind_renderbuffer(
            web_sys::WebGlRenderingContext::RENDERBUFFER,
            Some(&self.buffer),
        )
    }

    pub fn unbind(&mut self) {
        self.assert_bound_and_unbind();
        self.ctx
            .gl
            .bind_renderbuffer(web_sys::WebGlRenderingContext::RENDERBUFFER, None)
    }

    pub fn storage_for_depth(&mut self, width: i32, height: i32) {
        self.assert_bound();
        self.ctx.gl.renderbuffer_storage(
            web_sys::WebGlRenderingContext::RENDERBUFFER,
            web_sys::WebGlRenderingContext::DEPTH_COMPONENT16,
            width,
            height,
        )
    }

    fn assert_bound(&self) {
        assert_eq!(*self.ctx.bound_renderbuffer.borrow(), self.id);
    }

    fn assert_bound_and_unbind(&mut self) {
        assert_eq!(*self.ctx.bound_renderbuffer.borrow(), self.id);
        *self.ctx.bound_renderbuffer.borrow_mut() = 0;
    }

    fn assert_unbound(&self) {
        assert_ne!(*self.ctx.bound_renderbuffer.borrow(), self.id);
    }

    fn assert_unbound_and_bind(&mut self) {
        assert_ne!(*self.ctx.bound_renderbuffer.borrow(), self.id);
        *self.ctx.bound_renderbuffer.borrow_mut() = self.id;
    }
}

impl<'a> std::ops::Drop for Renderbuffer {
    fn drop(&mut self) {
        self.assert_unbound();
        log::debug!("deleting renderbuffer");
        self.ctx.gl.delete_renderbuffer(Some(&self.buffer));
    }
}

pub struct Texture2D {
    ctx: Rc<Context>,
    id: u32,
    texture: web_sys::WebGlTexture,
}

impl Texture2D {
    pub fn create(ctx: &Rc<Context>) -> Result<Self, JsValue> {
        let texture = ctx
            .gl
            .create_texture()
            .ok_or_else(|| JsValue::from_str("create_texture error"))?;
        let id = ctx.next_object_id();
        Ok(Texture2D {
            ctx: ctx.clone(),
            id,
            texture,
        })
    }

    pub fn bind(&mut self) {
        self.assert_unbound_and_bind();
        self.ctx.gl.bind_texture(
            web_sys::WebGlRenderingContext::TEXTURE_2D,
            Some(&self.texture),
        )
    }

    pub fn unbind(&mut self) {
        self.assert_bound_and_unbind();
        self.ctx
            .gl
            .bind_texture(web_sys::WebGlRenderingContext::TEXTURE_2D, None)
    }

    pub fn tex_image_2d(
        &mut self,
        width: i32,
        height: i32,
        pixels: Option<&[u8]>,
    ) -> Result<(), JsValue> {
        self.assert_bound();
        self.ctx
            .gl
            .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                web_sys::WebGlRenderingContext::TEXTURE_2D,
                0,
                web_sys::WebGlRenderingContext::RGBA as i32,
                width,
                height,
                0,
                web_sys::WebGlRenderingContext::RGBA,
                web_sys::WebGlRenderingContext::UNSIGNED_BYTE,
                pixels,
            )
    }

    fn assert_bound(&self) {
        assert_eq!(*self.ctx.bound_texture.borrow(), self.id);
    }

    fn assert_bound_and_unbind(&mut self) {
        assert_eq!(*self.ctx.bound_texture.borrow(), self.id);
        *self.ctx.bound_texture.borrow_mut() = 0;
    }

    fn assert_unbound(&self) {
        assert_ne!(*self.ctx.bound_texture.borrow(), self.id);
    }

    fn assert_unbound_and_bind(&mut self) {
        assert_ne!(*self.ctx.bound_texture.borrow(), self.id);
        *self.ctx.bound_texture.borrow_mut() = self.id;
    }
}

impl std::ops::Drop for Texture2D {
    fn drop(&mut self) {
        self.assert_unbound();
        log::debug!("deleting texture");
        self.ctx.gl.delete_texture(Some(&self.texture));
    }
}
