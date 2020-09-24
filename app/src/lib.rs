#![no_implicit_prelude]

mod engine;
mod game;

use ::std::cell::RefCell;
use ::std::clone::Clone;
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

use engine::opengl::Context;
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
    game: Rc<RefCell<Game>>,
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
        let game = Rc::new(RefCell::new(Game::new(&ctx)?));
        let engine = engine::Engine::new(&ctx, game.clone());
        Ok(Console { engine, game })
    }

    pub fn start(&mut self) -> Result<(), JsValue> {
        ::log::info!("wasmgame starting");
        self.engine
            .register_event_handler("click", self.game.clone())?;
        self.engine.start()
    }
}
