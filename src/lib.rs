mod game;

use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

struct AnimatedCanvas {
    window: web_sys::Window,
    canvas: web_sys::HtmlCanvasElement,
    context: web_sys::CanvasRenderingContext2d,
    offscreen_canvas: web_sys::HtmlCanvasElement,
    offscreen_context: web_sys::CanvasRenderingContext2d,

    gc: game::Canvas,

    drawing: bool,
    changed: bool,

    last_render: std::time::Duration,
}

impl AnimatedCanvas {
    fn new(element_id: &str) -> Self {
        let window = web_sys::window().expect("cannot get window object");
        let document = window.document().expect("cannot get document object");

        let canvas = document
            .get_element_by_id(element_id)
            .expect("cannot find canvas element")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("element not of type canvas");
        let context = canvas
            .get_context("2d")
            .expect("getContext failed")
            .expect("unsupported context type")
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .expect("context 2D of unexpected type");

        let offscreen_canvas = document
            .create_element("canvas")
            .expect("cannot create canvas element")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("element not of type canvas");
        offscreen_canvas.set_width(canvas.width());
        offscreen_canvas.set_height(canvas.height());
        let offscreen_context = offscreen_canvas
            .get_context("2d")
            .expect("getContext failed")
            .expect("unsupported context type")
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .expect("context 2D of unexpected type");

        let gc = game::Canvas::new(offscreen_canvas.width(), offscreen_canvas.height());

        Self {
            window,
            canvas,
            context,
            offscreen_canvas,
            offscreen_context,
            gc,
            drawing: false,
            changed: false,
            last_render: std::time::Duration::from_secs(0),
        }
    }
}

impl game::Renderer for AnimatedCanvas {
    fn prep(&mut self, _millis: f64) -> Result<(), JsValue> {
        self.gc.fill(game::Color::White);
        for x in 0..self.gc.width() {
            for y in 0..self.gc.height() {
                self.gc.draw_pixel(x, y, game::Color::RGB(0, 150, 30));
            }
        }
        for x in 0..self.gc.width() {
            self.gc.draw_pixel(x, 0, game::Color::Black);
            self.gc
                .draw_pixel(x, self.gc.height() - 1, game::Color::Black);
        }
        for y in 0..self.gc.height() {
            self.gc.draw_pixel(0, y, game::Color::Black);
            self.gc
                .draw_pixel(self.gc.width() - 1, y, game::Color::Black);
        }
        self.offscreen_context
            .put_image_data(self.gc.image_data(), 0.0, 0.0)
            .unwrap();

        game::stroke_hexatile(&self.offscreen_context, 100, 100, 30, 26);
        game::stroke_hexatile(&self.offscreen_context, 100 + 30 + 15, 100 + 26, 30, 26);
        game::stroke_hexatile(&self.offscreen_context, 100 + 30 + 15, 100 - 26, 30, 26);
        game::stroke_hexatile(&self.offscreen_context, 100, 100 - 26 * 2, 30, 26);
        game::stroke_hexatile(&self.offscreen_context, 100, 100 + 26 * 2, 30, 26);
        game::stroke_hexatile(&self.offscreen_context, 100 - 30 - 15, 100 + 26, 30, 26);
        game::stroke_hexatile(&self.offscreen_context, 100 - 30 - 15, 100 - 26, 30, 26);
        Ok(())
    }

    fn render(&mut self, millis: f64) -> Result<(), JsValue> {
        self.context
            .draw_image_with_html_canvas_element(&self.offscreen_canvas, 0.0, 0.0)
            .unwrap();
        self.last_render = std::time::Duration::from_micros((millis * 1000.0) as u64);
        Ok(())
    }

    fn ready(&self) -> bool {
        true
    }

    fn done(&self) -> bool {
        self.last_render >= std::time::Duration::from_secs(3)
    }
}

#[wasm_bindgen(start)]
pub fn main() -> Result<(), wasm_bindgen::JsValue> {
    wasm_logger::init(wasm_logger::Config::default());
    log::info!("wasmgame loading");

    let r = Rc::new(RefCell::new(AnimatedCanvas::new("wasmgame")));
    let e = game::Engine::new(r);
    log::info!("wasmgame starting");
    e.start()
}
