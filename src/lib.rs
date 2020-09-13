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
use std::{vec, vec::Vec};

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
attribute vec3 normal;

uniform mat4 mvp;
uniform mat4 normals;

varying vec3 lighting;

void main() {
    gl_Position = mvp * vec4(position, 1.0);

    vec3 ambientLightColor = vec3(0.4, 0.4, 0.4);
    vec3 directionalLightColor = vec3(1.0, 1.0, 0.8);
    vec4 directionalLight = vec4(-2, 3, 4, 0.0);

    float intensity = max(dot(normals * vec4(normal, 0.0), normalize(directionalLight)), 0.0);
    lighting = ambientLightColor + (directionalLightColor * intensity);
}
"#;

const FRAGMENT_SHADER: &str = r#"
#version 100

varying highp vec3 lighting;

void main() {
    highp vec4 baseColor = vec4(0.2, 0.2, 0.2, 1.0);
    gl_FragColor = vec4(baseColor.rgb * lighting, baseColor.a);
}
"#;

impl game::Renderer for AnimatedCanvas {
    fn setup(&mut self, ctx: &game::RenderingContext) -> Result<(), JsValue> {
        ctx.gl.enable(web_sys::WebGlRenderingContext::CULL_FACE);
        ctx.gl.hint(
            web_sys::WebGlRenderingContext::GENERATE_MIPMAP_HINT,
            web_sys::WebGlRenderingContext::NICEST,
        );

        let vertex_shader = ctx.create_vertex_shader(VERTEX_SHADER)?;
        let fragment_shader = ctx.create_fragment_shader(FRAGMENT_SHADER)?;
        let program = ctx.link_program(&vertex_shader, &fragment_shader)?;

        ctx.gl.use_program(Some(&program));

        let mat_model = game::math::Mat4::with_array([
            1.0, 0.0, 0.0, 0.0, //br
            0.0, 1.0, 0.0, 0.0, //br
            0.0, 0.0, 1.0, 0.0, //br
            0.0, 0.0, 0.0, 1.0, //br
        ]);
        let eye_pos = game::math::Vec3::new(4.0, 3.0, -3.0);
        let center_pos = game::math::Vec3::new(0.0, 0.0, 0.0);
        let up_direction = game::math::Vec3::new(0.0, 1.0, 0.0);
        let mat_view = game::math::look_at(&eye_pos, &center_pos, &up_direction);
        let mat_projection = game::math::project(40.0, 4.0 / 3.0, 0.1, 100.0); // ->90deg

        let mat_model_view = &mat_view * &mat_model;
        let mat_mvp = &mat_projection * &mat_model_view;
        let mat_normals = match mat_model_view.invert() {
            Some(inv) => inv.transpose(),
            None => {
                log::error!("mat_model_view not invertible");
                // TODO: use only 3x3
                mat_model_view
            }
        };

        let mvp = ctx
            .gl
            .get_uniform_location(&program, "mvp")
            .ok_or_else(|| JsValue::from_str("get_uniform_location mvp error"))?;
        ctx.gl
            .uniform_matrix4fv_with_f32_array(Some(&mvp), false, mat_mvp.slice());

        let normals = ctx
            .gl
            .get_uniform_location(&program, "normals")
            .ok_or_else(|| JsValue::from_str("get_uniform_location normals error"))?;
        ctx.gl
            .uniform_matrix4fv_with_f32_array(Some(&normals), false, mat_normals.slice());

        let loc_position = ctx.gl.get_attrib_location(&program, "position");
        if loc_position == -1 {
            return Err(JsValue::from_str("position attribute not defined"));
        }
        let loc_normal = ctx.gl.get_attrib_location(&program, "normal");
        if loc_normal == -1 {
            return Err(JsValue::from_str("normal attribute not defined"));
        }

        let mut interleaved: Vec<f32> = vec![0.0; models::HEXATILE_INDICES.len() * 6];
        game::interleave_with_normals(
            &models::HEXATILE_INDICES,
            &models::HEXATILE_VERTICES,
            &mut interleaved,
        );

        let hexatile_vao = ctx
            .vertex_array_object_ext
            .create_vertex_array_oes()
            .ok_or_else(|| JsValue::from_str("create_vertex_array_oes vao error"))?;
        ctx.vertex_array_object_ext
            .bind_vertex_array_oes(Some(&hexatile_vao));
        let hexatile_buffer = ctx
            .gl
            .create_buffer()
            .ok_or_else(|| JsValue::from_str("create_buffer hexatile_buffer error"))?;
        ctx.gl.bind_buffer(
            web_sys::WebGlRenderingContext::ARRAY_BUFFER,
            Some(&hexatile_buffer),
        );
        unsafe {
            let view = js_sys::Float32Array::view(&interleaved);
            ctx.gl.buffer_data_with_array_buffer_view(
                web_sys::WebGlRenderingContext::ARRAY_BUFFER,
                &view,
                web_sys::WebGlRenderingContext::STATIC_DRAW,
            );
        }
        ctx.gl.vertex_attrib_pointer_with_i32(
            loc_position as u32,
            3,
            web_sys::WebGlRenderingContext::FLOAT,
            false,
            4 * 3 * 2,
            0,
        );
        ctx.gl.vertex_attrib_pointer_with_i32(
            loc_normal as u32,
            3,
            web_sys::WebGlRenderingContext::FLOAT,
            false,
            4 * 3 * 2,
            4 * 3,
        );
        ctx.gl.enable_vertex_attrib_array(loc_position as u32);
        ctx.gl.enable_vertex_attrib_array(loc_normal as u32);
        ctx.gl
            .bind_buffer(web_sys::WebGlRenderingContext::ARRAY_BUFFER, None);
        ctx.vertex_array_object_ext.bind_vertex_array_oes(None);

        // clear

        ctx.gl.clear_color(0.9, 0.9, 0.9, 1.0);
        ctx.gl
            .clear(web_sys::WebGlRenderingContext::COLOR_BUFFER_BIT);

        // draw

        ctx.vertex_array_object_ext
            .bind_vertex_array_oes(Some(&hexatile_vao));
        ctx.gl.draw_arrays(
            web_sys::WebGlRenderingContext::TRIANGLES,
            0,
            models::HEXATILE_INDICES.len() as i32,
        );
        ctx.vertex_array_object_ext.bind_vertex_array_oes(None);

        Ok(())
    }

    fn render(&mut self, _ctx: &game::RenderingContext, millis: f64) -> Result<(), JsValue> {
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
