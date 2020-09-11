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
use std::option::{Option::None, Option::Some};
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

const VERTEX_SHADER: &str = r#"
#version 100

attribute vec4 position;

void main() {
    gl_Position = position;
}
"#;

const FRAGMENT_SHADER: &str = r#"
#version 100

void main() {
    gl_FragColor = vec4(0.5, 0.5, 0.5, 1.0);
}
"#;

impl game::Renderer for AnimatedCanvas {
    fn setup(&mut self, ctx: &game::RenderingContext) -> Result<(), JsValue> {
        let vertex_shader = ctx.create_vertex_shader(VERTEX_SHADER).unwrap();
        let fragment_shader = ctx.create_fragment_shader(FRAGMENT_SHADER).unwrap();
        let program = ctx.link_program(&vertex_shader, &fragment_shader).unwrap();

        ctx.gl.use_program(Some(&program));

        let vao = ctx
            .vertex_array_object_ext
            .create_vertex_array_oes()
            .unwrap();
        let vbo = ctx.gl.create_buffer().unwrap();
        let ebo = ctx.gl.create_buffer().unwrap();

        ctx.vertex_array_object_ext
            .bind_vertex_array_oes(Some(&vao));

        // CCW
        // 2----3
        // |\   |
        // |  \ |
        // 0----1
        let vertices: [f32; 12] = [
            -0.9, -0.9, 0.0, //br
            0.7, -0.9, 0.0, //br
            -0.7, 0.7, 0.0, //br
            0.5, 0.5, 0.0, //br
        ];
        let indices: [u8; 6] = [0, 1, 2, 1, 3, 2];

        ctx.gl
            .bind_buffer(web_sys::WebGlRenderingContext::ARRAY_BUFFER, Some(&vbo));

        unsafe {
            let view = js_sys::Float32Array::view(&vertices);
            ctx.gl.buffer_data_with_array_buffer_view(
                web_sys::WebGlRenderingContext::ARRAY_BUFFER,
                &view,
                web_sys::WebGlRenderingContext::STATIC_DRAW,
            );
        }
        ctx.gl.bind_buffer(
            web_sys::WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,
            Some(&ebo),
        );
        unsafe {
            let view = js_sys::Uint8Array::view(&indices);
            ctx.gl.buffer_data_with_array_buffer_view(
                web_sys::WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,
                &view,
                web_sys::WebGlRenderingContext::STATIC_DRAW,
            );
        }

        ctx.gl.vertex_attrib_pointer_with_i32(
            0,
            3,
            web_sys::WebGlRenderingContext::FLOAT,
            false,
            0,
            0,
        );
        ctx.gl.enable_vertex_attrib_array(0);

        ctx.gl.clear_color(1.0, 1.0, 1.0, 1.0);
        ctx.gl
            .clear(web_sys::WebGlRenderingContext::COLOR_BUFFER_BIT);

        ctx.gl.draw_elements_with_i32(
            web_sys::WebGlRenderingContext::TRIANGLES,
            indices.len() as i32,
            web_sys::WebGlRenderingContext::UNSIGNED_BYTE,
            0,
        );

        ctx.vertex_array_object_ext.bind_vertex_array_oes(None);

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
