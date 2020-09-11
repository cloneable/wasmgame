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
use std::ops::FnMut;
use std::option::{Option, Option::None, Option::Some};
use std::rc::Rc;
use std::result::{Result, Result::Ok};

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

type RequestAnimationFrameCallback = Closure<dyn FnMut(f64) + 'static>;

fn request_animation_frame_helper(callback: Option<&RequestAnimationFrameCallback>) {
    web_sys::window()
        .unwrap()
        .request_animation_frame(callback.unwrap().as_ref().unchecked_ref())
        .unwrap();
}

pub struct Engine {
    ctx: RenderingContext,
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
            .dyn_into::<web_sys::OesVertexArrayObject>()
            .unwrap();
        let instanced_arrays_ext = gl
            .get_extension("ANGLE_instanced_arrays")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::AngleInstancedArrays>()
            .unwrap();
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
                return;
            }

            self.renderer.borrow_mut().render(&self.ctx, millis).unwrap();

            let c0 = c.clone();
            request_animation_frame_helper(c0.borrow().as_ref());
        }) as Box<dyn FnMut(f64) + 'static>));

        request_animation_frame_helper(callback.borrow().as_ref());
        Ok(())
    }
}
