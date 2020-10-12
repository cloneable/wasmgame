use ::std::boxed::Box;
use ::std::cell::RefCell;
use ::std::clone::Clone;
use ::std::convert::AsRef;
use ::std::ops::FnMut;
use ::std::option::{Option, Option::None, Option::Some};
use ::std::rc::Rc;
use ::std::result::{Result, Result::Err, Result::Ok};

use ::wasm_bindgen::closure::Closure;
use ::wasm_bindgen::JsCast;

use super::time::{Framerate, Time};
use super::Error;

pub trait LoopHandler {
    fn init(&mut self) -> Result<(), Error> {
        Ok(())
    }
    fn update(&mut self, t: Time) -> Result<(), Error>;
    fn draw(&mut self, t: Time) -> Result<bool, Error>;
    fn done(&self) -> bool {
        false
    }
}

type RequestAnimationFrameCallback = Closure<dyn FnMut(f64) + 'static>;

pub struct Loop {
    window: ::web_sys::Window,
    handler: Rc<RefCell<dyn LoopHandler>>,
    framerate: RefCell<Framerate>,
}

impl Loop {
    pub fn new(
        window: &::web_sys::Window, handler: Rc<RefCell<dyn LoopHandler>>,
    ) -> Rc<Self> {
        Rc::new(Loop {
            window: window.clone(),
            handler,
            framerate: RefCell::new(Framerate::new()),
        })
    }

    pub fn start(self: &Rc<Self>) -> Result<(), Error> {
        self.handler.borrow_mut().init()?;
        // Part of this is taken from the wasm-bindgen guide.
        // This kinda works for now, but needs to be checked for
        // leaks.
        // TODO: Check if handler, callback instances not freed.
        // TODO: See if there's a better/cleaner way to do this.
        let callback = Rc::new(RefCell::new(
            None as Option<RequestAnimationFrameCallback>,
        ));
        let c0 = callback.clone();
        let self0 = self.clone();
        *callback.borrow_mut() =
            Some(Closure::wrap(Box::new(move |millis: f64| {
                if self0.handler.borrow().done() {
                    ::log::debug!(
                        "framerate: {:?}",
                        self0.framerate.borrow().rate()
                    );
                    let _ = c0.borrow_mut().take();
                    ::log::info!("wasmgame ending");
                    return;
                }

                let t = Time::from_millis(millis);
                match self0.handler.borrow_mut().draw(t) {
                    Ok(true) => {
                        self0.framerate.borrow_mut().record_timestamp(t)
                    }
                    Ok(false) => (),
                    Err(error) => ::log::error!("{:?}", error),
                }

                let self1 = self0.clone();
                let c1 = c0.clone();
                ::wasm_bindgen_futures::spawn_local(
                    self1.prepare_next_frame(c1, t),
                );
            })
                as Box<dyn FnMut(f64) + 'static>));

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
        callback: Rc<RefCell<Option<RequestAnimationFrameCallback>>>, t: Time,
    ) {
        self.handler.borrow_mut().update(t).unwrap();
        self.request_animation_frame_helper(callback.borrow().as_ref());
    }

    fn request_animation_frame_helper(
        self: Rc<Self>, callback: Option<&RequestAnimationFrameCallback>,
    ) {
        self.window
            .request_animation_frame(callback.unwrap().as_ref().unchecked_ref())
            .unwrap();
    }
}
