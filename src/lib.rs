#![no_implicit_prelude]

extern crate js_sys;
extern crate log;
extern crate std;
extern crate wasm_bindgen;
extern crate wasm_bindgen_macro;
extern crate wasm_logger;
extern crate web_sys;

mod engine;
mod game;

use std::cell::RefCell;
use std::clone::Clone;
use std::default::Default;
use std::mem::drop;
use std::rc::Rc;
use std::result::{Result, Result::Ok};

use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_macro::wasm_bindgen;

use engine::opengl::Context;
use game::Game;

#[wasm_bindgen(start)]
pub fn wasm_main() -> Result<(), JsValue> {
    wasm_logger::init(wasm_logger::Config::default());
    log::info!("wasmgame init");
    Ok(())
}

#[wasm_bindgen]
pub struct Console {
    game: Rc<RefCell<Game>>,
}

#[wasm_bindgen]
impl Console {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let game = Rc::new(RefCell::new(Game::new()));
        Console { game }
    }

    pub fn start(&self) -> Result<(), JsValue> {
        log::info!("wasmgame loading");

        let window = web_sys::window().expect("cannot get window object");
        let document = window.document().expect("cannot get document object");
        let canvas = document
            .get_element_by_id("wasmgame")
            .expect("cannot find canvas element")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("element not of type canvas");

        let ctx = Context::from_canvas(&canvas)?;
        let e = engine::Engine::new(ctx, self.game.clone());
        log::info!("wasmgame starting");
        e.start()
    }
}
