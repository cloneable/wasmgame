use ::std::boxed::Box;
use ::std::cell::RefCell;
use ::std::clone::Clone;
use ::std::convert::AsRef;
use ::std::ops::FnMut;
use ::std::option::{Option, Option::Some};
use ::std::rc::Rc;
use ::std::result::{Result, Result::Ok};

use ::wasm_bindgen::closure::Closure;
use ::wasm_bindgen::JsCast;
use ::wasm_bindgen::JsValue;
use ::web_sys::Event;
use ::web_sys::EventTarget;
use ::web_sys::TouchEvent;

pub struct Listener {
    target: EventTarget,
    type_: &'static str,
    closure: Rc<RefCell<Option<Closure<dyn FnMut(&Event)>>>>,
}

impl Listener {
    pub fn new<E, F>(
        target: &EventTarget,
        type_: &'static str,
        callback: F,
    ) -> Result<Self, JsValue>
    where
        E: JsCast,
        F: FnMut(&E) + 'static,
    {
        let mut cb = callback;
        let closure = Rc::new(RefCell::new(Some(Closure::wrap(
            Box::new(move |e: &Event| {
                e.prevent_default();
                cb(e.unchecked_ref::<E>());
            }) as Box<dyn FnMut(&Event) + 'static>,
        ))));
        target.add_event_listener_with_callback_and_add_event_listener_options(
            type_,
            closure.borrow().as_ref().unwrap().as_ref().unchecked_ref(),
            ::web_sys::AddEventListenerOptions::new().passive(false),
        )?;
        Ok(Listener {
            target: target.clone(),
            type_,
            closure,
        })
    }
}

impl ::std::ops::Drop for Listener {
    fn drop(&mut self) {
        let closure = self.closure.borrow_mut().take();
        self.target
            .remove_event_listener_with_callback(
                self.type_,
                closure.as_ref().unwrap().as_ref().unchecked_ref(),
            )
            .unwrap()
    }
}

pub(crate) struct TouchEventWrapper<'a>(&'a TouchEvent);

impl TouchEventWrapper<'_> {
    pub(crate) fn wrap(e: &TouchEvent) -> TouchEventWrapper<'_> {
        TouchEventWrapper(e)
    }
}

impl ::std::fmt::Debug for TouchEventWrapper<'_> {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> Result<(), ::std::fmt::Error> {
        f.write_str("all=")?;
        let mut l = f.debug_list();
        let touches = self.0.touches();
        for i in 0..touches.length() {
            if let Some(touch) = touches.item(i) {
                l.entry(&touch.identifier());
            }
        }
        l.finish()?;
        f.write_str(", target=")?;
        let mut l = f.debug_list();
        let touches = self.0.target_touches();
        for i in 0..touches.length() {
            if let Some(touch) = touches.item(i) {
                l.entry(&touch.identifier());
            }
        }
        l.finish()?;
        f.write_str(", changed=")?;
        let mut l = f.debug_list();
        let touches = self.0.changed_touches();
        for i in 0..touches.length() {
            if let Some(touch) = touches.item(i) {
                l.entry(&touch.identifier());
            }
        }
        l.finish()
    }
}
