#![no_implicit_prelude]

extern crate js_sys;
extern crate log;
extern crate std;
extern crate wasm_bindgen;
extern crate wasm_bindgen_macro;
extern crate wasm_logger;
extern crate web_sys;

mod game;
mod meshes;
mod opengl;
mod scene;
mod shaders;

use std::cell::RefCell;
use std::clone::Clone;
use std::default::Default;
use std::mem::drop;
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

impl game::Renderer for AnimatedCanvas {
    fn setup(&mut self, ctx: &opengl::Context) -> Result<(), JsValue> {
        let mut cam = scene::Camera::new();
        cam.set_position(0.5, 1.4, 3.0)
            .set_frustum(35.0, 4.0 / 3.0, 0.1, 100.0)
            .refresh();

        let mut hexatile = scene::Model::new(&meshes::HEXATILE_VERTICES, &meshes::HEXATILE_INDICES);
        hexatile.add_instance(game::math::Mat4::with_array([
            1.0, 0.0, 0.0, 0.0, //br
            0.0, 3.0, 0.0, 0.0, //br
            0.0, 0.0, 1.0, 0.0, //br
            -0.6, 0.0, 0.0, 1.0, //br
        ]));
        hexatile.add_instance(game::math::Mat4::with_array([
            1.0, 0.0, 0.0, 0.0, //br
            0.0, 2.0, 0.0, 0.0, //br
            0.0, 0.0, 1.0, 0.0, //br
            0.0, 0.0, 0.0, 1.0, //br
        ]));
        hexatile.add_instance(game::math::Mat4::with_array([
            1.0, 0.0, 0.0, 0.0, //br
            0.0, 1.0, 0.0, 0.0, //br
            0.0, 0.0, 1.0, 0.0, //br
            0.6, 0.0, 0.0, 1.0, //br
        ]));
        hexatile.update_normals(&cam);

        // ===== OpenGL setup =====

        let vertex_shader = ctx.create_vertex_shader(shaders::HEXATILE_VERTEX_SHADER)?;
        let fragment_shader = ctx.create_fragment_shader(shaders::HEXATILE_FRAGMENT_SHADER)?;
        let program = ctx.link_program(&vertex_shader, &fragment_shader)?;

        ctx.gl.use_program(Some(&program));

        // ===== Uniforms =====

        let loc_view = ctx
            .gl
            .get_uniform_location(&program, "view")
            .ok_or_else(|| JsValue::from_str("get_uniform_location error: view"))?;
        ctx.gl
            .uniform_matrix4fv_with_f32_array(Some(&loc_view), false, cam.view_matrix().slice());
        let loc_projection = ctx
            .gl
            .get_uniform_location(&program, "projection")
            .ok_or_else(|| JsValue::from_str("get_uniform_location error: projection"))?;
        ctx.gl.uniform_matrix4fv_with_f32_array(
            Some(&loc_projection),
            false,
            cam.projection_matrix().slice(),
        );

        // ===== Attributes =====

        let loc_position = ctx.gl.get_attrib_location(&program, "position");
        if loc_position == -1 {
            return Err(JsValue::from_str("position attribute not defined"));
        }
        let loc_normal = ctx.gl.get_attrib_location(&program, "normal");
        if loc_normal == -1 {
            return Err(JsValue::from_str("normal attribute not defined"));
        }
        let loc_model = ctx.gl.get_attrib_location(&program, "model");
        if loc_model == -1 {
            return Err(JsValue::from_str("model attribute not defined"));
        }
        let loc_normals = ctx.gl.get_attrib_location(&program, "normals");
        if loc_normals == -1 {
            return Err(JsValue::from_str("normals attribute not defined"));
        }

        // ===== VAO =====

        let vao_hexatile = ctx
            .vertex_array_object_ext
            .create_vertex_array_oes()
            .ok_or_else(|| JsValue::from_str("create_vertex_array_oes vao error"))?;
        ctx.vertex_array_object_ext
            .bind_vertex_array_oes(Some(&vao_hexatile));

        // ===== vertices =====

        let _ = opengl::ArrayBuffer::new(ctx)?
            .bind()
            .set_buffer_data(&hexatile.vertices)
            .set_vertex_attribute_pointer_vec3(loc_position)
            .unbind();
        let _ = opengl::ArrayBuffer::new(ctx)?
            .bind()
            .set_buffer_data(&hexatile.normals)
            .set_vertex_attribute_pointer_vec3(loc_normal)
            .unbind();
        let _ = opengl::ArrayBuffer::new(ctx)?
            .bind()
            .set_buffer_data(&hexatile.instance_model_data)
            .set_vertex_attribute_pointer_mat4(loc_model)
            .set_vertex_attrib_divisor_mat4(loc_model, 1)
            .unbind();
        let _ = opengl::ArrayBuffer::new(ctx)?
            .bind()
            .set_buffer_data(&hexatile.instance_normals_data)
            .set_vertex_attribute_pointer_mat4(loc_normals)
            .set_vertex_attrib_divisor_mat4(loc_normals, 1)
            .unbind();

        ctx.gl.enable_vertex_attrib_array(loc_position as u32);
        ctx.gl.enable_vertex_attrib_array(loc_normal as u32);
        for i in 0..=3 {
            ctx.gl.enable_vertex_attrib_array(loc_model as u32 + i);
            ctx.gl.enable_vertex_attrib_array(loc_normals as u32 + i);
        }

        ctx.vertex_array_object_ext.bind_vertex_array_oes(None);

        // clear

        ctx.gl.clear_color(0.7, 0.7, 0.7, 1.0);
        ctx.gl
            .clear(web_sys::WebGlRenderingContext::COLOR_BUFFER_BIT);

        // draw

        ctx.vertex_array_object_ext
            .bind_vertex_array_oes(Some(&vao_hexatile));

        ctx.instanced_arrays_ext.draw_arrays_instanced_angle(
            web_sys::WebGlRenderingContext::TRIANGLES,
            0,
            hexatile.vertices.len() as i32 / 3,
            hexatile.instances.len() as i32,
        );

        ctx.vertex_array_object_ext.bind_vertex_array_oes(None);

        Ok(())
    }

    fn render(&mut self, _ctx: &opengl::Context, millis: f64) -> Result<(), JsValue> {
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
    log::info!("wasmgame init");
    Ok(())
}

#[wasm_bindgen]
pub struct Game {
    renderer: Rc<RefCell<dyn game::Renderer>>,
}

#[wasm_bindgen]
impl Game {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let renderer = Rc::new(RefCell::new(AnimatedCanvas::new()));
        Game { renderer }
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

        let ctx = opengl::Context::from_canvas(&canvas)?;
        let e = game::Engine::new(ctx, self.renderer.clone());
        log::info!("wasmgame starting");
        e.start()
    }
}
