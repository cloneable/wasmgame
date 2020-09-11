#![no_implicit_prelude]

extern crate js_sys;
extern crate log;
extern crate std;
extern crate wasm_bindgen;
extern crate wasm_bindgen_macro;
extern crate wasm_logger;
extern crate web_sys;

mod game;

use std::cell::RefCell;
use std::default::Default;
use std::rc::Rc;
use std::result::{Result, Result::Ok};
use std::time::Duration;

use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_macro::wasm_bindgen;

struct AnimatedCanvas {
    last_render: Duration,
}

impl AnimatedCanvas {
    fn new() -> Self {
        Self {
            last_render: Duration::from_secs(0),
        }
    }
}

impl game::Renderer for AnimatedCanvas {
    fn setup(&mut self, ctx: &game::RenderingContext) -> Result<(), JsValue> {
        Ok(())
    }

    fn render(&mut self, ctx: &game::RenderingContext, millis: f64) -> Result<(), JsValue> {
        self.last_render = Duration::from_micros((millis * 1000.0) as u64);
        Ok(())
    }

    fn done(&self) -> bool {
        self.last_render >= Duration::from_secs(3)
    }
}

#[wasm_bindgen(start)]
pub fn wasm_main() -> Result<(), JsValue> {
    wasm_logger::init(wasm_logger::Config::default());
    log::info!("wasmgame loading");

    let window = web_sys::window().expect("cannot get window object");
    let document = window.document().expect("cannot get document object");

    let canvas = document
        .get_element_by_id("wasmgame")
        .expect("cannot find canvas element")
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .expect("element not of type canvas");
    let gl = canvas
        .get_context("webgl")
        .expect("getContext failed")
        .expect("unsupported context type")
        .dyn_into::<web_sys::WebGlRenderingContext>()
        .expect("context of unexpected type");

    let r = Rc::new(RefCell::new(AnimatedCanvas::new()));
    let e = game::Engine::new(gl, r);
    log::info!("wasmgame starting");
    e.start()
}
