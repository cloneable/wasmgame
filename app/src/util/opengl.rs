use ::log;
use ::std::cell::Cell;
use ::std::clone::Clone;
use ::std::convert::From;
use ::std::convert::Into;
use ::std::option::{Option, Option::None, Option::Some};
use ::std::rc::Rc;
use ::std::result::{Result, Result::Err, Result::Ok};
use ::std::string::String;

use ::wasm_bindgen::JsCast;
use ::wasm_bindgen::JsValue;
use ::web_sys::WebGl2RenderingContext as WebGL;

pub struct Context {
    /// https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.HtmlCanvasElement.html
    pub canvas: ::web_sys::HtmlCanvasElement,
    /// https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.WebGl2RenderingContext.html
    pub gl: WebGL,

    next_object_id: Cell<u32>,
    bound_vertex_array_buffer: BindingTracker,
    bound_array_buffer: BindingTracker,
    bound_uniform_buffer: BindingTracker,
    bound_framebuffer: BindingTracker,
    bound_renderbuffer: BindingTracker,
    bound_texture: BindingTracker,
}

impl Context {
    pub fn from_canvas(
        canvas: &::web_sys::HtmlCanvasElement,
    ) -> Result<Self, JsValue> {
        let gl = canvas
            .get_context_with_context_options(
                "webgl2",
                &::web_sys::WebGlContextAttributes::new(),
            )
            .expect("getContext failed")
            .expect("unsupported context type")
            .dyn_into::<WebGL>()
            .expect("context of unexpected type");
        // TODO: find better place for this. some init func?
        gl.enable(WebGL::CULL_FACE);
        gl.enable(WebGL::DEPTH_TEST);
        gl.hint(WebGL::GENERATE_MIPMAP_HINT, WebGL::NICEST);
        Ok(Context {
            canvas: canvas.clone(),
            gl,
            next_object_id: Cell::new(1),
            bound_vertex_array_buffer: BindingTracker::new(),
            bound_array_buffer: BindingTracker::new(),
            bound_uniform_buffer: BindingTracker::new(),
            bound_framebuffer: BindingTracker::new(),
            bound_renderbuffer: BindingTracker::new(),
            bound_texture: BindingTracker::new(),
        })
    }

    pub fn resized(&self) {
        self.gl.viewport(0, 0, self.width(), self.height())
    }

    pub fn next_object_id(&self) -> u32 {
        let id = self.next_object_id.get();
        self.next_object_id.set(id + 1);
        id
    }

    pub fn width(&self) -> i32 {
        self.canvas.width() as i32
    }

    pub fn height(&self) -> i32 {
        self.canvas.height() as i32
    }
}

pub struct VertexArrayObject {
    ctx: Rc<Context>,
    id: u32,
    vao: ::web_sys::WebGlVertexArrayObject,
}

impl VertexArrayObject {
    pub fn create(ctx: &Rc<Context>) -> Result<Self, JsValue> {
        let vao = ctx.gl.create_vertex_array().ok_or_else(|| {
            JsValue::from_str("create_vertex_array vao error")
        })?;
        let id = ctx.next_object_id();
        Ok(VertexArrayObject {
            ctx: ctx.clone(),
            id,
            vao,
        })
    }

    pub fn bind(&mut self) -> &mut Self {
        self.ctx
            .bound_vertex_array_buffer
            .assert_unbound_then_bind(self.id);
        self.ctx.gl.bind_vertex_array(Some(&self.vao));
        self
    }

    pub fn unbind(&mut self) -> &mut Self {
        self.ctx
            .bound_vertex_array_buffer
            .assert_bound_then_unbind(self.id);
        self.ctx.gl.bind_vertex_array(None);
        self
    }
}

impl ::std::ops::Drop for VertexArrayObject {
    fn drop(&mut self) {
        log::debug!("deleting vao");
        self.ctx.bound_vertex_array_buffer.assert_unbound(self.id);
        self.ctx.gl.delete_vertex_array(Some(&self.vao));
    }
}

pub struct ArrayBuffer {
    ctx: Rc<Context>,
    id: u32,
    buffer: ::web_sys::WebGlBuffer,
}

impl ArrayBuffer {
    pub fn create(ctx: &Rc<Context>) -> Result<Self, JsValue> {
        let buffer = ctx.gl.create_buffer().ok_or_else(|| {
            JsValue::from_str("create_buffer vbo_vertices error")
        })?;
        let id = ctx.next_object_id();
        Ok(ArrayBuffer {
            ctx: ctx.clone(),
            id,
            buffer,
        })
    }

    pub fn bind(&mut self) -> &mut Self {
        self.ctx
            .bound_array_buffer
            .assert_unbound_then_bind(self.id);
        self.ctx
            .gl
            .bind_buffer(WebGL::ARRAY_BUFFER, Some(&self.buffer));
        self
    }

    pub fn unbind(&mut self) {
        self.ctx
            .bound_array_buffer
            .assert_bound_then_unbind(self.id);
        self.ctx.gl.bind_buffer(WebGL::ARRAY_BUFFER, None);
    }

    pub fn allocate_dynamic(&mut self, size: usize) -> &mut Self {
        self.ctx.bound_array_buffer.assert_bound(self.id);
        self.ctx.gl.buffer_data_with_i32(
            WebGL::ARRAY_BUFFER,
            size as i32,
            WebGL::DYNAMIC_DRAW,
        );
        self
    }

    pub fn set_buffer_data(&mut self, data: &[f32]) -> &mut Self {
        self.ctx.bound_array_buffer.assert_bound(self.id);
        #[allow(unsafe_code)]
        unsafe {
            let view = ::js_sys::Float32Array::view(data);
            self.ctx.gl.buffer_data_with_array_buffer_view(
                WebGL::ARRAY_BUFFER,
                &view,
                WebGL::STATIC_DRAW,
            );
        }
        self
    }

    pub fn set_buffer_sub_data(
        &mut self, offset: i32, data: &[f32],
    ) -> &mut Self {
        self.ctx.bound_array_buffer.assert_bound(self.id);
        #[allow(unsafe_code)]
        unsafe {
            let view = ::js_sys::Float32Array::view(data);
            self.ctx.gl.buffer_sub_data_with_i32_and_array_buffer_view(
                WebGL::ARRAY_BUFFER,
                offset,
                &view,
            );
        }
        self
    }

    pub fn set_vertex_attribute_pointer_vec3(
        &mut self, attribute: Attribute, stride: usize, offset: usize,
    ) -> &mut Self {
        self.ctx.bound_array_buffer.assert_bound(self.id);
        self.ctx.gl.vertex_attrib_pointer_with_i32(
            attribute.0,
            3,
            WebGL::FLOAT,
            false,
            stride as i32 * 4,
            offset as i32 * 4,
        );
        self
    }

    pub fn set_vertex_attribute_pointer_mat4(
        &mut self, attribute: Attribute,
    ) -> &mut Self {
        self.ctx.bound_array_buffer.assert_bound(self.id);
        for i in 0..attribute.1 {
            self.ctx.gl.vertex_attrib_pointer_with_i32(
                attribute.0 + i as u32,
                4,
                WebGL::FLOAT,
                false,
                16 * 4,
                i as i32 * 4 * 4,
            );
        }
        self
    }

    pub fn set_vertex_attrib_divisor(
        &mut self, attribute: Attribute, divisor: usize,
    ) -> &mut Self {
        self.ctx.bound_array_buffer.assert_bound(self.id);
        for i in 0..attribute.1 {
            self.ctx
                .gl
                .vertex_attrib_divisor(attribute.0 + i as u32, divisor as u32);
        }
        self
    }
}

impl ::std::ops::Drop for ArrayBuffer {
    fn drop(&mut self) {
        log::debug!("deleting buffer");
        self.ctx.bound_array_buffer.assert_unbound(self.id);
        self.ctx.gl.delete_buffer(Some(&self.buffer));
    }
}

pub struct Uniform {
    ctx: Rc<Context>,
    location: ::web_sys::WebGlUniformLocation,
}

impl Uniform {
    pub fn find(
        ctx: &Rc<Context>, program: &Program, name: &str,
    ) -> Result<Self, JsValue> {
        let location = ctx
            .gl
            .get_uniform_location(&program.program, name)
            .ok_or_else(|| {
                JsValue::from_str("get_uniform_location error: view")
            })?;
        Ok(Uniform {
            ctx: ctx.clone(),
            location,
        })
    }

    pub fn set_mat4(&mut self, data: &[f32]) {
        self.ctx.gl.uniform_matrix4fv_with_f32_array(
            Some(&self.location),
            false,
            data,
        );
    }
}

#[derive(Copy, Clone, Debug)]
pub struct UniformBinding(u32);

impl UniformBinding {
    pub fn new(index: u32) -> Self {
        use ::std::panic;
        ::std::debug_assert!(index < 48);
        UniformBinding(index)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct UniformIndex(u32);

pub struct UniformBuffer {
    ctx: Rc<Context>,
    id: u32,
    buffer: ::web_sys::WebGlBuffer,
}

impl UniformBuffer {
    pub fn create(ctx: &Rc<Context>) -> Result<Self, JsValue> {
        let buffer = ctx.gl.create_buffer().ok_or_else(|| {
            JsValue::from_str("UniformBuffer: create_buffer error")
        })?;
        let id = ctx.next_object_id();
        Ok(UniformBuffer {
            ctx: ctx.clone(),
            id,
            buffer,
        })
    }

    pub fn bind(&mut self) -> &mut Self {
        self.ctx
            .bound_uniform_buffer
            .assert_unbound_then_bind(self.id);
        self.ctx
            .gl
            .bind_buffer(WebGL::UNIFORM_BUFFER, Some(&self.buffer));
        self
    }

    pub fn unbind(&mut self) {
        self.ctx
            .bound_uniform_buffer
            .assert_bound_then_unbind(self.id);
        self.ctx.gl.bind_buffer(WebGL::UNIFORM_BUFFER, None);
    }

    pub fn bind_buffer_base(&mut self, binding: UniformBinding) -> &mut Self {
        self.ctx.bound_uniform_buffer.assert_bound(self.id);
        self.ctx.gl.bind_buffer_base(
            WebGL::UNIFORM_BUFFER,
            binding.0,
            Some(&self.buffer),
        );
        self
    }

    pub fn bind_buffer_range(
        &mut self, binding: UniformBinding, offset: usize, size: usize,
    ) -> &mut Self {
        self.ctx.bound_uniform_buffer.assert_bound(self.id);
        self.ctx.gl.bind_buffer_range_with_i32_and_i32(
            WebGL::UNIFORM_BUFFER,
            binding.0,
            Some(&self.buffer),
            offset as i32,
            size as i32,
        );
        self
    }

    pub fn allocate_dynamic(&mut self, size: usize) -> &mut Self {
        self.ctx.bound_uniform_buffer.assert_bound(self.id);
        self.ctx.gl.buffer_data_with_i32(
            WebGL::UNIFORM_BUFFER,
            size as i32,
            WebGL::DYNAMIC_DRAW,
        );
        self
    }

    pub fn set_buffer_sub_data(
        &mut self, offset: i32, data: &[f32],
    ) -> &mut Self {
        self.ctx.bound_uniform_buffer.assert_bound(self.id);
        #[allow(unsafe_code)]
        unsafe {
            let view = ::js_sys::Float32Array::view(data);
            self.ctx.gl.buffer_sub_data_with_i32_and_array_buffer_view(
                WebGL::UNIFORM_BUFFER,
                offset,
                &view,
            );
        }
        self
    }
}

impl ::std::ops::Drop for UniformBuffer {
    fn drop(&mut self) {
        log::debug!("deleting uniform buffer");
        self.ctx.bound_uniform_buffer.assert_unbound(self.id);
        self.ctx.gl.delete_buffer(Some(&self.buffer));
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

    #[allow(dead_code)]
    pub fn disable(&self, ctx: &Rc<Context>) {
        for i in 0..self.1 {
            ctx.gl.disable_vertex_attrib_array(self.0 + i as u32);
        }
    }
}

pub struct Program {
    ctx: Rc<Context>,
    pub program: ::web_sys::WebGlProgram,
}

impl Program {
    pub fn create(ctx: &Rc<Context>) -> Result<Self, JsValue> {
        let program = ctx.gl.create_program().ok_or_else(|| {
            JsValue::from_str("Unable to create program object")
        })?;
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
            .get_program_parameter(&self.program, WebGL::LINK_STATUS)
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

    pub fn get_uniform_block_index(&self, name: &str) -> Option<UniformIndex> {
        let index = self.ctx.gl.get_uniform_block_index(&self.program, name);
        if index == WebGL::INVALID_INDEX {
            None
        } else {
            Some(UniformIndex(index))
        }
    }

    pub fn set_uniform_block_binding(
        &mut self, index: UniformIndex, binding: UniformBinding,
    ) {
        self.ctx
            .gl
            .uniform_block_binding(&self.program, index.0, binding.0);
    }
}

impl ::std::ops::Drop for Program {
    fn drop(&mut self) {
        log::debug!("deleting program");
        self.ctx.gl.delete_program(Some(&self.program));
    }
}

pub struct Shader {
    ctx: Rc<Context>,
    shader: ::web_sys::WebGlShader,
}

pub enum ShaderType {
    Vertex,
    Fragment,
}

impl Shader {
    pub fn create(
        ctx: &Rc<Context>, type_: ShaderType,
    ) -> Result<Self, JsValue> {
        let pt = match type_ {
            ShaderType::Vertex => WebGL::VERTEX_SHADER,
            ShaderType::Fragment => WebGL::FRAGMENT_SHADER,
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
            .get_shader_parameter(&self.shader, WebGL::COMPILE_STATUS)
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

impl ::std::ops::Drop for Shader {
    fn drop(&mut self) {
        log::debug!("deleting shader");
        self.ctx.gl.delete_shader(Some(&self.shader));
    }
}

pub struct Framebuffer {
    ctx: Rc<Context>,
    id: u32,
    buffer: ::web_sys::WebGlFramebuffer,
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
        self.ctx.bound_framebuffer.assert_unbound_then_bind(self.id);
        self.ctx
            .gl
            .bind_framebuffer(WebGL::FRAMEBUFFER, Some(&self.buffer))
    }

    pub fn unbind(&mut self) {
        self.ctx.bound_framebuffer.assert_bound_then_unbind(self.id);
        self.ctx.gl.bind_framebuffer(WebGL::FRAMEBUFFER, None)
    }

    pub fn check_status(&self) -> u32 {
        self.ctx.bound_framebuffer.assert_bound(self.id);
        self.ctx.gl.check_framebuffer_status(WebGL::FRAMEBUFFER)
    }

    pub fn texture2d_as_colorbuffer(&mut self, texture: &Texture2D) {
        self.ctx.bound_framebuffer.assert_bound(self.id);
        self.ctx.gl.framebuffer_texture_2d(
            WebGL::FRAMEBUFFER,
            WebGL::COLOR_ATTACHMENT0,
            WebGL::TEXTURE_2D,
            Some(&texture.texture),
            0,
        )
    }

    pub fn renderbuffer_as_depthbuffer(&mut self, buffer: &Renderbuffer) {
        self.ctx.bound_framebuffer.assert_bound(self.id);
        self.ctx.gl.framebuffer_renderbuffer(
            WebGL::FRAMEBUFFER,
            WebGL::DEPTH_ATTACHMENT,
            WebGL::RENDERBUFFER,
            Some(&buffer.buffer),
        )
    }

    pub fn read_pixels(
        &self, x: i32, y: i32, width: i32, height: i32, format: u32,
        type_: u32, pixels: &mut [u8],
    ) -> Result<(), JsValue> {
        self.ctx.bound_framebuffer.assert_bound(self.id);
        self.ctx.gl.read_pixels_with_opt_u8_array(
            x,
            y,
            width,
            height,
            format,
            type_,
            Some(pixels),
        )
    }
}

impl ::std::ops::Drop for Framebuffer {
    fn drop(&mut self) {
        log::debug!("deleting framebuffer");
        self.ctx.bound_framebuffer.assert_unbound(self.id);
        self.ctx.gl.delete_framebuffer(Some(&self.buffer));
    }
}

pub struct Renderbuffer {
    ctx: Rc<Context>,
    id: u32,
    buffer: ::web_sys::WebGlRenderbuffer,
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
        self.ctx
            .bound_renderbuffer
            .assert_unbound_then_bind(self.id);
        self.ctx
            .gl
            .bind_renderbuffer(WebGL::RENDERBUFFER, Some(&self.buffer))
    }

    pub fn unbind(&mut self) {
        self.ctx
            .bound_renderbuffer
            .assert_bound_then_unbind(self.id);
        self.ctx.gl.bind_renderbuffer(WebGL::RENDERBUFFER, None)
    }

    pub fn storage_for_depth(&mut self, width: i32, height: i32) {
        self.ctx.bound_renderbuffer.assert_bound(self.id);
        self.ctx.gl.renderbuffer_storage(
            WebGL::RENDERBUFFER,
            WebGL::DEPTH_COMPONENT16,
            width,
            height,
        )
    }
}

impl ::std::ops::Drop for Renderbuffer {
    fn drop(&mut self) {
        log::debug!("deleting renderbuffer");
        self.ctx.bound_vertex_array_buffer.assert_unbound(self.id);
        self.ctx.gl.delete_renderbuffer(Some(&self.buffer));
    }
}

pub struct Texture2D {
    ctx: Rc<Context>,
    id: u32,
    texture: ::web_sys::WebGlTexture,
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
        self.ctx.bound_texture.assert_unbound_then_bind(self.id);
        self.ctx
            .gl
            .bind_texture(WebGL::TEXTURE_2D, Some(&self.texture))
    }

    pub fn unbind(&mut self) {
        self.ctx.bound_texture.assert_bound_then_unbind(self.id);
        self.ctx.gl.bind_texture(WebGL::TEXTURE_2D, None)
    }

    pub fn tex_image_2d(
        &mut self, width: i32, height: i32, pixels: Option<&[u8]>,
    ) -> Result<(), JsValue> {
        self.ctx.bound_texture.assert_bound(self.id);
        self.ctx
            .gl
            .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                WebGL::TEXTURE_2D,
                0,
                WebGL::RGBA as i32,
                width,
                height,
                0,
                WebGL::RGBA,
                WebGL::UNSIGNED_BYTE,
                pixels,
            )
    }
}

impl ::std::ops::Drop for Texture2D {
    fn drop(&mut self) {
        log::debug!("deleting texture2d");
        self.ctx.bound_texture.assert_unbound(self.id);
        self.ctx.gl.delete_texture(Some(&self.texture));
    }
}

struct BindingTracker {
    #[cfg(debug_assertions)]
    bound_id: Cell<u32>,
}

impl BindingTracker {
    fn new() -> Self {
        BindingTracker {
            #[cfg(debug_assertions)]
            bound_id: Cell::new(0),
        }
    }

    #[cfg(debug_assertions)]
    fn assert_bound(&self, id: u32) {
        use ::std::panic;
        ::std::debug_assert_eq!(self.bound_id.get(), id);
    }

    #[cfg(not(debug_assertions))]
    fn assert_bound(&self, _id: u32) {}

    #[cfg(debug_assertions)]
    fn assert_bound_then_unbind(&self, id: u32) {
        use ::std::panic;
        let bound_id = self.bound_id.get();
        ::std::debug_assert_eq!(bound_id, id);
        self.bound_id.set(0);
    }

    #[cfg(not(debug_assertions))]
    fn assert_bound_then_unbind(&self, _id: u32) {}

    #[cfg(debug_assertions)]
    fn assert_unbound(&self, id: u32) {
        use ::std::panic;
        ::std::debug_assert_ne!(self.bound_id.get(), id);
    }

    #[cfg(not(debug_assertions))]
    fn assert_unbound(&self, _id: u32) {}

    #[cfg(debug_assertions)]
    fn assert_unbound_then_bind(&self, id: u32) {
        use ::std::panic;
        let bound_id = self.bound_id.get();
        ::std::debug_assert_ne!(bound_id, id);
        self.bound_id.set(id);
    }

    #[cfg(not(debug_assertions))]
    fn assert_unbound_then_bind(&self, _id: u32) {}
}
