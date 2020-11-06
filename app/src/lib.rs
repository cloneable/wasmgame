#![no_implicit_prelude]
#![cfg_attr(not(debug_assertions), allow(dead_code, unused_macros))]

mod engine;
mod game;
mod util;

use ::std::{
    cell::RefCell,
    clone::Clone,
    convert::Into,
    mem::drop,
    rc::Rc,
    result::{Result, Result::Ok},
};

use ::wasm_bindgen;
use ::wasm_bindgen::{JsCast, JsValue};
use ::wasm_bindgen_macro::wasm_bindgen;
use ::web_sys;

use crate::{
    game::Game,
    util::{event, opengl::Context},
};

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: ::wee_alloc::WeeAlloc = ::wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(start)]
pub fn wasm_main() -> Result<(), JsValue> {
    #[cfg(feature = "console_error_panic_hook")]
    ::std::panic::set_hook(::std::boxed::Box::new(
        ::console_error_panic_hook::hook,
    ));
    ::wasm_logger::init(::wasm_logger::Config::new(::log::Level::Trace));
    ::log::info!("wasmgame init");
    Ok(())
}

#[wasm_bindgen]
pub struct Console {
    engine_loop: Rc<engine::core::Loop>,
    _game: Rc<RefCell<Game>>,
    _on_resize: event::Listener,
    _on_click: event::Listener,
    _on_mousedown: event::Listener,
    _on_mouseup: event::Listener,
    _on_mousemove: event::Listener,
    _on_touchstart: event::Listener,
    _on_touchmove: event::Listener,
    _on_touchend: event::Listener,
    _on_touchcancel: event::Listener,
    _on_webglcontextlost: event::Listener,
    _on_webglcontextrestored: event::Listener,
    _on_webglcontextcreationerror: event::Listener,
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
        let engine_loop = engine::core::Loop::new(&window, _game.clone());

        let game0 = _game.clone();
        let _on_resize =
            event::Listener::new(&window, "resize", move |event| {
                game0.borrow_mut().on_resize(event)
            })?;
        let game0 = _game.clone();
        let _on_click =
            event::Listener::new(&ctx.canvas, "click", move |event| {
                game0.borrow_mut().on_click(event)
            })?;
        let game0 = _game.clone();
        let _on_mousedown =
            event::Listener::new(&ctx.canvas, "mousedown", move |event| {
                game0.borrow_mut().on_mousedown(event)
            })?;
        let game0 = _game.clone();
        let _on_mousemove =
            event::Listener::new(&ctx.canvas, "mousemove", move |event| {
                game0.borrow_mut().on_mousemove(event)
            })?;
        let game0 = _game.clone();
        let _on_mouseup =
            event::Listener::new(&window, "mouseup", move |event| {
                game0.borrow_mut().on_mouseup(event)
            })?;
        let game0 = _game.clone();
        let _on_touchstart =
            event::Listener::new(&ctx.canvas, "touchstart", move |event| {
                game0.borrow_mut().on_touchstart(event)
            })?;
        let game0 = _game.clone();
        let _on_touchmove =
            event::Listener::new(&ctx.canvas, "touchmove", move |event| {
                game0.borrow_mut().on_touchmove(event)
            })?;
        let game0 = _game.clone();
        let _on_touchend =
            event::Listener::new(&ctx.canvas, "touchend", move |event| {
                game0.borrow_mut().on_touchend(event)
            })?;
        let game0 = _game.clone();
        let _on_touchcancel =
            event::Listener::new(&ctx.canvas, "touchcancel", move |event| {
                game0.borrow_mut().on_touchcancel(event)
            })?;
        let game0 = _game.clone();
        let _on_webglcontextlost = event::Listener::new(
            &ctx.canvas,
            "webglcontextlost",
            move |event| game0.borrow_mut().on_webglcontextlost(event),
        )?;
        let game0 = _game.clone();
        let _on_webglcontextrestored = event::Listener::new(
            &ctx.canvas,
            "webglcontextrestored",
            move |event| game0.borrow_mut().on_webglcontextrestored(event),
        )?;
        let game0 = _game.clone();
        let _on_webglcontextcreationerror = event::Listener::new(
            &ctx.canvas,
            "webglcontextcreationerror",
            move |event| game0.borrow_mut().on_webglcontextcreationerror(event),
        )?;
        Ok(Console {
            engine_loop,
            _game,
            _on_resize,
            _on_click,
            _on_mousedown,
            _on_mouseup,
            _on_mousemove,
            _on_touchstart,
            _on_touchmove,
            _on_touchend,
            _on_touchcancel,
            _on_webglcontextlost,
            _on_webglcontextrestored,
            _on_webglcontextcreationerror,
        })
    }

    pub fn start(&mut self) -> Result<(), JsValue> {
        ::log::info!("wasmgame starting");
        self.engine_loop.start().map_err(Into::<JsValue>::into)
    }
}
