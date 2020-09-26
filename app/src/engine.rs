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
    fn update(&mut self, t: Time) -> Result<(), Error>;
    fn render(&mut self) -> Result<(), Error>;
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
    ctx: Rc<opengl::Context>,
    renderer: Rc<RefCell<dyn Renderer>>,
    callbacks: RefCell<Vec<Rc<RefCell<EventCallback>>>>,
    framerate: RefCell<Framerate>,
}

impl Engine {
    pub fn new(ctx: &Rc<opengl::Context>, renderer: Rc<RefCell<dyn Renderer>>) -> Rc<Self> {
        Rc::new(Engine {
            ctx: ctx.clone(),
            renderer,
            callbacks: RefCell::new(Vec::new()),
            framerate: RefCell::new(Framerate::new()),
        })
    }

    pub fn register_event_handler<T: ::wasm_bindgen::JsCast + 'static>(
        self: &Rc<Self>,
        type_: &'static str,
        listener: Rc<RefCell<dyn EventHandler<T>>>,
    ) -> Result<(), Error> {
        let c = Rc::new(RefCell::new(Closure::wrap(
            Box::new(move |event: &::web_sys::Event| {
                listener.borrow_mut().handle(
                    Time::from_millis(event.time_stamp()),
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
        // Part of this is taken from the wasm-bindgen guide.
        // This kinda works for now, but needs to be checked for
        // leaks.
        // TODO: Check if renderer, callback instances not freed.
        // TODO: See if there's a better/cleaner way to do this.
        let callback = Rc::new(RefCell::new(None as Option<RequestAnimationFrameCallback>));
        let c0 = callback.clone();
        let self0 = self.clone();
        *callback.borrow_mut() = Some(Closure::wrap(Box::new(move |millis: f64| {
            if self0.renderer.borrow().done() {
                ::log::debug!("framerate: {:?}", self0.framerate.borrow().rate());
                let _ = c0.borrow_mut().take();
                ::log::info!("wasmgame ending");
                return;
            }

            let t = Time::from_millis(millis);
            self0.framerate.borrow_mut().record_timestamp(t);
            self0.renderer.borrow_mut().render().unwrap();

            let self1 = self0.clone();
            let c1 = c0.clone();
            ::wasm_bindgen_futures::spawn_local(self1.prepare_next_frame(c1, t));
        }) as Box<dyn FnMut(f64) + 'static>));

        // first frame always gets timestamp=0.
        // TODO: or just pass performance.now()?
        ::wasm_bindgen_futures::spawn_local(
            self.clone()
                .prepare_next_frame(callback, Time::from_millis(0.0)),
        );
        Ok(())
    }

    // TODO: replace with requestPostAnimationFrame() once available.
    async fn prepare_next_frame(
        self: Rc<Self>,
        callback: Rc<RefCell<Option<RequestAnimationFrameCallback>>>,
        t: Time,
    ) {
        self.renderer.borrow_mut().update(t).unwrap();
        request_animation_frame_helper(callback.borrow().as_ref());
    }
}

pub trait EventHandler<T: ::wasm_bindgen::JsCast + 'static> {
    fn handle(&mut self, t: Time, event: &T);
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

struct Framerate {
    buf: [Time; 32],
    index: usize,
}

impl Framerate {
    fn new() -> Self {
        Framerate {
            buf: [::std::default::Default::default(); 32],
            index: 0,
        }
    }

    fn record_timestamp(&mut self, t: Time) {
        self.buf[self.index] = t;
        self.index = (self.index + 1) % self.buf.len();
    }

    fn rate(&self) -> Rate {
        let len = self.buf.len();
        let first = self.buf[self.index];
        let last = self.buf[(self.index - 1 + len) % len];
        len / (last - first)
    }
}

#[derive(Copy, Clone, Default, PartialOrd, PartialEq, Debug)]
pub struct Time(f64);

#[derive(Copy, Clone, Default, PartialOrd, PartialEq, Debug)]
pub struct Duration(f64);

#[derive(Copy, Clone, Default, PartialOrd, PartialEq, Debug)]
pub struct Rate(f64);

impl Time {
    pub fn from_millis(millis: f64) -> Self {
        Time(millis)
    }
}

impl ::std::ops::Sub for Time {
    type Output = Duration;
    fn sub(self, other: Time) -> Duration {
        Duration(self.0 - other.0)
    }
}

impl ::std::ops::Div<Duration> for usize {
    type Output = Rate;
    fn div(self, d: Duration) -> Rate {
        Rate(self as f64 * 1000.0 / d.0)
    }
}
