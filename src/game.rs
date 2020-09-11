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
use std::ops::FnMut;
use std::option::{Option, Option::None, Option::Some};
use std::rc::Rc;
use std::result::{Result, Result::Err, Result::Ok};
use std::string::String;

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
    pub fn create_vertex_shader(&self, glsl: &str) -> Result<web_sys::WebGlShader, String> {
        self.create_shader(glsl, web_sys::WebGlRenderingContext::VERTEX_SHADER)
    }

    pub fn create_fragment_shader(&self, glsl: &str) -> Result<web_sys::WebGlShader, String> {
        self.create_shader(glsl, web_sys::WebGlRenderingContext::FRAGMENT_SHADER)
    }

    fn create_shader(&self, glsl: &str, type_: u32) -> Result<web_sys::WebGlShader, String> {
        let shader = self.gl.create_shader(type_).unwrap();
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
            Err(self
                .gl
                .get_shader_info_log(&shader)
                .unwrap_or_else(|| String::from("Unknown error creating shader")))
        }
    }

    pub fn link_program(
        &self,
        vertex_shader: &web_sys::WebGlShader,
        fragment_shader: &web_sys::WebGlShader,
    ) -> Result<web_sys::WebGlProgram, String> {
        let program = self
            .gl
            .create_program()
            .ok_or_else(|| String::from("Unable to create shader object"))?;

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
            Err(self
                .gl
                .get_program_info_log(&program)
                .unwrap_or_else(|| String::from("Unknown error creating program object")))
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
