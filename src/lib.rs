#![no_implicit_prelude]

extern crate js_sys;
extern crate log;
extern crate std;
extern crate wasm_bindgen;
extern crate wasm_bindgen_macro;
extern crate wasm_logger;
extern crate web_sys;

mod game;
mod math;
mod meshes;
mod opengl;
mod scene;
mod shaders;

use std::cell::RefCell;
use std::clone::Clone;
use std::default::Default;
use std::mem::drop;
use std::option::Option::Some;
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
    fn setup(&mut self, ctx: &opengl::Context) -> Result<(), JsValue> {
        let mut cam = scene::Camera::new();
        cam.set_position(0.5, 1.4, 3.0)
            .set_frustum(35.0, 4.0 / 3.0, 0.1, 100.0)
            .refresh();

        let mut hexatile = scene::Model::new(&meshes::HEXATILE_VERTICES, &meshes::HEXATILE_INDICES);
        hexatile.add_instance(math::Mat4::with_array([
            1.0, 0.0, 0.0, 0.0, //br
            0.0, 3.0, 0.0, 0.0, //br
            0.0, 0.0, 1.0, 0.0, //br
            -0.6, 0.0, 0.0, 1.0, //br
        ]));
        hexatile.add_instance(math::Mat4::with_array([
            1.0, 0.0, 0.0, 0.0, //br
            0.0, 2.0, 0.0, 0.0, //br
            0.0, 0.0, 1.0, 0.0, //br
            0.0, 0.0, 0.0, 1.0, //br
        ]));
        hexatile.add_instance(math::Mat4::with_array([
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

        let mut loc_view = opengl::Uniform::find(ctx, &program, "view")?;
        loc_view.set_mat4(cam.view_matrix().slice());
        let mut loc_projection = opengl::Uniform::find(ctx, &program, "projection")?;
        loc_projection.set_mat4(cam.projection_matrix().slice());

        // ===== Attributes =====

        let mut loc_position = opengl::Attribute::find(ctx, &program, "position", 1)?;
        let mut loc_normal = opengl::Attribute::find(ctx, &program, "normal", 1)?;
        let mut loc_model = opengl::Attribute::find(ctx, &program, "model", 4)?;
        let mut loc_normals = opengl::Attribute::find(ctx, &program, "normals", 4)?;

        // ===== VAO =====

        let mut vao_hexatile = opengl::VertexArrayObject::create(ctx)?;
        vao_hexatile.bind();

        // ===== vertices =====

        let _ = opengl::ArrayBuffer::create(ctx)?
            .bind()
            .set_buffer_data(&hexatile.vertices)
            .set_vertex_attribute_pointer_vec3(&loc_position)
            .unbind();
        let _ = opengl::ArrayBuffer::create(ctx)?
            .bind()
            .set_buffer_data(&hexatile.normals)
            .set_vertex_attribute_pointer_vec3(&loc_normal)
            .unbind();
        let _ = opengl::ArrayBuffer::create(ctx)?
            .bind()
            .set_buffer_data(&hexatile.instance_model_data)
            .set_vertex_attribute_pointer_mat4(&loc_model)
            .set_vertex_attrib_divisor_mat4(&loc_model, 1)
            .unbind();
        let _ = opengl::ArrayBuffer::create(ctx)?
            .bind()
            .set_buffer_data(&hexatile.instance_normals_data)
            .set_vertex_attribute_pointer_mat4(&loc_normals)
            .set_vertex_attrib_divisor_mat4(&loc_normals, 1)
            .unbind();

        loc_position.enable();
        loc_normal.enable();
        loc_model.enable();
        loc_normals.enable();

        vao_hexatile.unbind();

        // clear

        ctx.gl.clear_color(0.7, 0.7, 0.7, 1.0);
        ctx.gl
            .clear(web_sys::WebGlRenderingContext::COLOR_BUFFER_BIT);

        // draw

        vao_hexatile.bind();

        ctx.instanced_arrays_ext.draw_arrays_instanced_angle(
            web_sys::WebGlRenderingContext::TRIANGLES,
            0,
            hexatile.vertices.len() as i32 / 3,
            hexatile.instances.len() as i32,
        );

        vao_hexatile.unbind();

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
