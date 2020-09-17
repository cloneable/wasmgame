extern crate js_sys;
extern crate log;
extern crate std;
extern crate wasm_bindgen;
extern crate wasm_bindgen_futures;
extern crate web_sys;

pub mod opengl;
pub mod math;
pub mod scene;
pub mod util;

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
    fn setup(&mut self, ctx: &opengl::Context) -> Result<(), JsValue>;
    fn render(&mut self, ctx: &opengl::Context, millis: f64) -> Result<(), JsValue>;
    fn done(&self) -> bool;
}

type RequestAnimationFrameCallback = Closure<dyn FnMut(f64) + 'static>;

fn request_animation_frame_helper(callback: Option<&RequestAnimationFrameCallback>) {
    web_sys::window()
        .unwrap()
        .request_animation_frame(callback.unwrap().as_ref().unchecked_ref())
        .unwrap();
}

pub struct Engine {
    pub ctx: opengl::Context,
    renderer: Rc<RefCell<dyn Renderer>>,
}

impl Engine {
    pub fn new(ctx: opengl::Context, renderer: Rc<RefCell<dyn Renderer>>) -> Rc<Self> {
        Rc::new(Self { ctx, renderer })
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
