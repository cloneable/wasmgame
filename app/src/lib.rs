#![no_implicit_prelude]

mod engine;
mod game;
mod util;

use ::std::cell::RefCell;
use ::std::clone::Clone;
use ::std::convert::Into;
use ::std::default::Default;
use ::std::mem::drop;
use ::std::rc::Rc;
use ::std::result::{Result, Result::Ok};

use ::wasm_bindgen;
use ::wasm_bindgen::JsCast;
use ::wasm_bindgen::JsValue;
use ::wasm_bindgen_macro::wasm_bindgen;
use ::wasm_logger;
use ::web_sys;

use crate::util::event;
use crate::util::opengl::Context;
use game::Game;

#[wasm_bindgen(start)]
pub fn wasm_main() -> Result<(), JsValue> {
    wasm_logger::init(wasm_logger::Config::default());
    ::log::info!("wasmgame init");
    Ok(())
}

#[wasm_bindgen]
pub struct Console {
    engine: Rc<engine::Engine>,
    _game: Rc<RefCell<Game>>,
    _on_click: event::Listener,
    _on_mousedown: event::Listener,
    _on_mouseup: event::Listener,
    _on_mousemove: event::Listener,
    _on_touchstart: event::Listener,
    _on_touchmove: event::Listener,
    _on_touchend: event::Listener,
    _on_touchcancel: event::Listener,
}

#[wasm_bindgen]
impl Console {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<Console, JsValue> {
        ::log::info!("wasmgame loading");
        let window = web_sys::window().expect("cannot get window object");
        let document = window.document().expect("cannot get document object");
        let canvas = document
            .get_element_by_id("wasmgame")
            .expect("cannot find canvas element")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("element not of type canvas");

        let ctx = Rc::new(Context::from_canvas(&canvas)?);
        let _game = Rc::new(RefCell::new(
            Game::new(&ctx).map_err(Into::<JsValue>::into)?,
        ));
        _game.borrow_mut().init().map_err(Into::<JsValue>::into)?;
        let engine = engine::Engine::new(_game.clone());
        let game0 = _game.clone();
        let _on_click = event::Listener::new(&ctx.canvas, "click", move |event| {
            game0.borrow_mut().on_click(event)
        })?;
        let game0 = _game.clone();
        let _on_mousedown = event::Listener::new(&ctx.canvas, "mousedown", move |event| {
            game0.borrow_mut().on_mousedown(event)
        })?;
        let game0 = _game.clone();
        let _on_mousemove = event::Listener::new(&ctx.canvas, "mousemove", move |event| {
            game0.borrow_mut().on_mousemove(event)
        })?;
        let game0 = _game.clone();
        let _on_mouseup = event::Listener::new(&window, "mouseup", move |event| {
            game0.borrow_mut().on_mouseup(event)
        })?;
        let game0 = _game.clone();
        let _on_touchstart = event::Listener::new(&ctx.canvas, "touchstart", move |event| {
            game0.borrow_mut().on_touchstart(event)
        })?;
        let game0 = _game.clone();
        let _on_touchmove = event::Listener::new(&ctx.canvas, "touchmove", move |event| {
            game0.borrow_mut().on_touchmove(event)
        })?;
        let game0 = _game.clone();
        let _on_touchend = event::Listener::new(&window, "touchend", move |event| {
            game0.borrow_mut().on_touchend(event)
        })?;
        let game0 = _game.clone();
        let _on_touchcancel = event::Listener::new(&window, "touchcancel", move |event| {
            game0.borrow_mut().on_touchcancel(event)
        })?;
        Ok(Console {
            engine,
            _game,
            _on_click,
            _on_mousedown,
            _on_mouseup,
            _on_mousemove,
            _on_touchstart,
            _on_touchmove,
            _on_touchend,
            _on_touchcancel,
        })
    }

    pub fn start(&mut self) -> Result<(), JsValue> {
        ::log::info!("wasmgame starting");
        self.engine.start().map_err(Into::<JsValue>::into)
    }
}
