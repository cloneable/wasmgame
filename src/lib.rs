#![no_implicit_prelude]

extern crate js_sys;
extern crate log;
extern crate std;
extern crate wasm_bindgen;
extern crate wasm_bindgen_macro;
extern crate wasm_logger;
extern crate web_sys;

mod game;
mod models;

use std::cell::RefCell;
use std::default::Default;
use std::option::{Option::None, Option::Some};
use std::rc::Rc;
use std::result::{Result, Result::Err, Result::Ok};
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

attribute vec3 position;

uniform vec3 eye_pos;
uniform vec3 center_pos;
uniform mat4 model;

mat4 look_at(const vec3 eye, const vec3 center, in vec3 up) {
    vec3 forward = normalize(center - eye);
    vec3 side = normalize(cross(forward, up));
    up = cross(side, forward);
    return mat4(
        vec4(side.x, up.x, -forward.x, 0),
        vec4(side.y, up.y, -forward.y, 0),
        vec4(side.z, up.z, -forward.z, 0),
        vec4(-dot(side,eye), -dot(up,eye), dot(forward, eye), 1)
    );
}

mat4 project(float fov, float near, float far) {
    float scale = 1.0 / tan(radians(fov) / 2.0);
    float d = -1.0 / (far - near);
    return mat4(
        vec4(scale, 0, 0, 0),
        vec4(0, scale, 0, 0),
        vec4(0, 0, (far + near) * d, -1),
        vec4(0, 0, far * near * d, 0)
    );
}

void main() {
    mat4 view = look_at(eye_pos, center_pos, vec3(0, 1, 0));
    mat4 projection = project(80.0, 0.1, 10.0);
    gl_Position = projection * view * model * vec4(position, 1.0);
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
        let vertex_shader = ctx.create_vertex_shader(VERTEX_SHADER)?;
        let fragment_shader = ctx.create_fragment_shader(FRAGMENT_SHADER)?;
        let program = ctx.link_program(&vertex_shader, &fragment_shader)?;

        ctx.gl.use_program(Some(&program));

        let model_mat = ctx
            .gl
            .get_uniform_location(&program, "model")
            .ok_or_else(|| JsValue::from_str("get_uniform_location model error"))?;
        ctx.gl.uniform_matrix4fv_with_f32_array(
            Some(&model_mat),
            false,
            &[
                0.5, 0.0, 0.0, 0.0, //br
                0.0, 0.2, 0.0, 0.0, //br
                0.0, 0.0, 0.5, 0.0, //br
                0.0, 0.0, 0.0, 1.0, //br
            ],
        );
        let eye_pos = ctx
            .gl
            .get_uniform_location(&program, "eye_pos")
            .ok_or_else(|| JsValue::from_str("get_uniform_location eye_pos error"))?;
        ctx.gl.uniform3f(Some(&eye_pos), 0.0, 1.0, -1.0);
        let center_pos = ctx
            .gl
            .get_uniform_location(&program, "center_pos")
            .ok_or_else(|| JsValue::from_str("get_uniform_location center_pos error"))?;
        ctx.gl.uniform3f(Some(&center_pos), 0.0, 0.0, 0.0);
        let position = ctx.gl.get_attrib_location(&program, "position");
        if position == -1 {
            return Err(JsValue::from_str("position attribute not defined"));
        }

        let vao = ctx
            .vertex_array_object_ext
            .create_vertex_array_oes()
            .ok_or_else(|| JsValue::from_str("create_vertex_array_oes vao error"))?;
        let vbo = ctx
            .gl
            .create_buffer()
            .ok_or_else(|| JsValue::from_str("create_buffer vbo error"))?;
        let ebo = ctx
            .gl
            .create_buffer()
            .ok_or_else(|| JsValue::from_str("create_buffer ebo error"))?;

        ctx.vertex_array_object_ext
            .bind_vertex_array_oes(Some(&vao));

        ctx.gl
            .bind_buffer(web_sys::WebGlRenderingContext::ARRAY_BUFFER, Some(&vbo));

        unsafe {
            let view = js_sys::Float32Array::view(&models::HEXATILE_VERTICES);
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
            let view = js_sys::Uint8Array::view(&models::HEXATILE_INDICES);
            ctx.gl.buffer_data_with_array_buffer_view(
                web_sys::WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,
                &view,
                web_sys::WebGlRenderingContext::STATIC_DRAW,
            );
        }

        ctx.gl.vertex_attrib_pointer_with_i32(
            position as u32,
            3,
            web_sys::WebGlRenderingContext::FLOAT,
            false,
            0,
            0,
        );
        ctx.gl.enable_vertex_attrib_array(0);

        ctx.gl.clear_color(0.9, 0.9, 0.9, 1.0);
        ctx.gl
            .clear(web_sys::WebGlRenderingContext::COLOR_BUFFER_BIT);

        ctx.gl.draw_elements_with_i32(
            web_sys::WebGlRenderingContext::TRIANGLES,
            models::HEXATILE_INDICES.len() as i32,
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
