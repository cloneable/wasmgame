extern crate js_sys;
extern crate log;
extern crate std;
extern crate wasm_bindgen;
extern crate wasm_bindgen_futures;
extern crate web_sys;

pub mod math;
pub mod opengl;
pub mod picker;
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
use std::vec::Vec;

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;

pub trait Renderer {
    fn setup(&mut self, ctx: &Rc<opengl::Context>) -> Result<(), JsValue>;
    fn render(&mut self, ctx: &Rc<opengl::Context>, millis: f64) -> Result<(), JsValue>;
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
    pub ctx: Rc<opengl::Context>,
    renderer: Rc<RefCell<dyn Renderer>>,
    callbacks: RefCell<Vec<Rc<RefCell<EventCallback>>>>,
}

impl Engine {
    pub fn new(ctx: &Rc<opengl::Context>, renderer: Rc<RefCell<dyn Renderer>>) -> Rc<Self> {
        Rc::new(Engine {
            ctx: ctx.clone(),
            renderer,
            callbacks: RefCell::new(Vec::new()),
        })
    }

    pub fn register_event_handler<T: wasm_bindgen::JsCast + 'static>(
        self: &Rc<Self>,
        type_: &'static str,
        listener: Rc<RefCell<dyn EventHandler<T>>>,
    ) -> Result<(), JsValue> {
        let self0 = self.clone();
        let c = Rc::new(RefCell::new(Closure::wrap(
            Box::new(move |event: &web_sys::Event| {
                listener.borrow_mut().handle(
                    &self0.ctx,
                    event.time_stamp(),
                    event.dyn_ref::<T>().unwrap(),
                );
            }) as Box<dyn FnMut(&web_sys::Event) + 'static>,
        )));
        {
            let handler = c.as_ref().borrow();
            self.ctx
                .gl
                .canvas()
                .unwrap()
                .unchecked_ref::<web_sys::HtmlCanvasElement>()
                .add_event_listener_with_callback(type_, handler.as_ref().unchecked_ref())?;
        }
        self.callbacks.borrow_mut().push(c.clone());
        Ok(())
    }

    pub fn start(self: &Rc<Self>) -> Result<(), JsValue> {
        self.renderer.borrow_mut().setup(&self.ctx)?;
        // Part of this is taken from the wasm-bindgen guide.
        // This kinda works for now, but needs to be checked for
        // leaks.
        // TODO: Check if renderer, callback instances not freed.
        // TODO: See if there's a better/cleaner way to do this.
        let callback = Rc::new(RefCell::new(None as Option<RequestAnimationFrameCallback>));
        let c = callback.clone();
        let self0 = self.clone();
        *callback.borrow_mut() = Some(Closure::wrap(Box::new(move |millis: f64| {
            if self0.renderer.borrow().done() {
                let _ = c.borrow_mut().take();
                log::info!("wasmgame ending");
                return;
            }

            self0
                .renderer
                .borrow_mut()
                .render(&self0.ctx, millis)
                .unwrap();

            let c0 = c.clone();
            request_animation_frame_helper(c0.borrow().as_ref());
        }) as Box<dyn FnMut(f64) + 'static>));

        request_animation_frame_helper(callback.borrow().as_ref());
        Ok(())
    }
}

pub trait EventHandler<T: wasm_bindgen::JsCast + 'static> {
    fn handle(&mut self, ctx: &opengl::Context, millis: f64, event: &T);
}

pub type EventCallback = Closure<dyn FnMut(&web_sys::Event) + 'static>;

pub mod attrib {
    use crate::engine::opengl::Attribute;

    pub const POSITION: Attribute = Attribute(0, 1);
    pub const NORMAL: Attribute = Attribute(1, 1);
    pub const INSTANCE_ID: Attribute = Attribute(3, 1);
    pub const MODEL: Attribute = Attribute(4, 4);
    pub const NORMALS: Attribute = Attribute(8, 4);
}
