pub mod picker;
pub mod scene;
pub mod util;

use crate::util::opengl;
use ::std::boxed::Box;
use ::std::cell::RefCell;
use ::std::clone::Clone;
use ::std::convert::AsRef;
use ::std::convert::From;
use ::std::convert::Into;
use ::std::ops::FnMut;
use ::std::option::{Option, Option::None, Option::Some};
use ::std::rc::Rc;
use ::std::result::{Result, Result::Ok};
use ::std::string::String;
use ::std::string::ToString;
use ::std::vec::Vec;

use ::wasm_bindgen::closure::Closure;
use ::wasm_bindgen::JsCast;
use ::wasm_bindgen::JsValue;

pub trait Renderer {
    fn setup(&mut self, ctx: &Rc<opengl::Context>) -> Result<(), Error>;
    fn render(&mut self, ctx: &Rc<opengl::Context>, millis: f64) -> Result<(), Error>;
    fn done(&self) -> bool;
}

type RequestAnimationFrameCallback = Closure<dyn FnMut(f64) + 'static>;

fn request_animation_frame_helper(callback: Option<&RequestAnimationFrameCallback>) {
    ::web_sys::window()
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

    pub fn register_event_handler<T: ::wasm_bindgen::JsCast + 'static>(
        self: &Rc<Self>,
        type_: &'static str,
        listener: Rc<RefCell<dyn EventHandler<T>>>,
    ) -> Result<(), Error> {
        let self0 = self.clone();
        let c = Rc::new(RefCell::new(Closure::wrap(
            Box::new(move |event: &::web_sys::Event| {
                listener.borrow_mut().handle(
                    &self0.ctx,
                    event.time_stamp(),
                    event.dyn_ref::<T>().unwrap(),
                );
            }) as Box<dyn FnMut(&::web_sys::Event) + 'static>,
        )));
        {
            let handler = c.as_ref().borrow();
            self.ctx
                .gl
                .canvas()
                .unwrap()
                .unchecked_ref::<::web_sys::HtmlCanvasElement>()
                .add_event_listener_with_callback(type_, handler.as_ref().unchecked_ref())?;
        }
        self.callbacks.borrow_mut().push(c.clone());
        Ok(())
    }

    pub fn start(self: &Rc<Self>) -> Result<(), Error> {
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
                ::log::info!("wasmgame ending");
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

pub trait EventHandler<T: ::wasm_bindgen::JsCast + 'static> {
    fn handle(&mut self, ctx: &opengl::Context, millis: f64, event: &T);
}

pub type EventCallback = Closure<dyn FnMut(&::web_sys::Event) + 'static>;

pub mod attrib {
    use crate::util::opengl::Attribute;

    pub const POSITION: Attribute = Attribute(0, 1);
    pub const NORMAL: Attribute = Attribute(1, 1);
    pub const INSTANCE_COLOR: Attribute = Attribute(2, 1);
    pub const INSTANCE_ID: Attribute = Attribute(3, 1);
    pub const MODEL: Attribute = Attribute(4, 4);
    pub const NORMALS: Attribute = Attribute(8, 4);
}

pub enum Error {
    Internal(String),
    JsValue(JsValue),
}

impl Error {
    pub fn new(msg: &str) -> Self {
        Error::Internal(msg.to_string())
    }
}

impl From<JsValue> for Error {
    fn from(value: JsValue) -> Error {
        Error::JsValue(value)
    }
}

impl Into<JsValue> for Error {
    fn into(self) -> JsValue {
        match self {
            Error::Internal(msg) => JsValue::from(msg),
            Error::JsValue(v) => v,
        }
    }
}

impl ::std::fmt::Debug for Error {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> Result<(), ::std::fmt::Error> {
        match self {
            Error::Internal(msg) => f.write_str(msg),
            Error::JsValue(v) => f.write_str(v.as_string().unwrap().as_str()),
        }
    }
}
