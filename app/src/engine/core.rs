use ::std::{
    boxed::Box,
    cell::RefCell,
    clone::Clone,
    convert::AsRef,
    ops::FnMut,
    option::{
        Option,
        Option::{None, Some},
    },
    rc::Rc,
    result::{
        Result,
        Result::{Err, Ok},
    },
};

use ::wasm_bindgen::{closure::Closure, JsCast};

use crate::engine::{
    time::{Framerate, Time},
    Error,
};

// TODO: Use event or sth else to terminate loop and drop LookHandler.
pub trait LoopHandler: super::Drawable {
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
                match self0.handler.borrow_mut().draw() {
                    Ok(_) => self0.framerate.borrow_mut().record_timestamp(t),
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
