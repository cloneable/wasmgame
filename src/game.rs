mod meshes;
mod shaders;

extern crate js_sys;
extern crate log;
extern crate std;
extern crate wasm_bindgen;
extern crate wasm_bindgen_macro;
extern crate wasm_logger;
extern crate web_sys;

use crate::engine;

use std::result::{Result, Result::Ok};
use std::time::Duration;

use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;

use engine::math::Mat4;
use engine::opengl::{ArrayBuffer, Context, VertexArrayObject};
use engine::scene::Camera;
use engine::scene::Model;

pub struct Game {
    last_render: Duration,
}

impl Game {
    pub fn new() -> Self {
        Self {
            last_render: Duration::from_secs(0),
        }
    }
}

impl engine::Renderer for Game {
    fn setup(&mut self, ctx: &Context) -> Result<(), JsValue> {
        let mut cam = Camera::new();
        cam.set_position(0.5, 1.4, 3.0)
            .set_frustum(35.0, 4.0 / 3.0, 0.1, 100.0)
            .refresh();

        let mut hexatile = Model::new(&meshes::HEXATILE_VERTICES, &meshes::HEXATILE_INDICES);
        hexatile.add_instance(Mat4::with_array([
            1.0, 0.0, 0.0, 0.0, //br
            0.0, 3.0, 0.0, 0.0, //br
            0.0, 0.0, 1.0, 0.0, //br
            -0.6, 0.0, 0.0, 1.0, //br
        ]));
        hexatile.add_instance(Mat4::with_array([
            1.0, 0.0, 0.0, 0.0, //br
            0.0, 2.0, 0.0, 0.0, //br
            0.0, 0.0, 1.0, 0.0, //br
            0.0, 0.0, 0.0, 1.0, //br
        ]));
        hexatile.add_instance(Mat4::with_array([
            1.0, 0.0, 0.0, 0.0, //br
            0.0, 1.0, 0.0, 0.0, //br
            0.0, 0.0, 1.0, 0.0, //br
            0.6, 0.0, 0.0, 1.0, //br
        ]));
        hexatile.update_normals(&cam);

        // ===== Program setup =====

        let mut program = shaders::HexatileProgram::new(ctx)?;
        program.activate();

        program.set_view(cam.view_matrix());
        program.set_projection(cam.projection_matrix());

        // ===== VAO =====

        let mut vao_hexatile = VertexArrayObject::create(ctx)?;
        vao_hexatile.bind();

        // ===== vertices =====

        let _ = ArrayBuffer::create(ctx)?
            .bind()
            .set_buffer_data(&hexatile.vertices)
            .set_vertex_attribute_pointer_vec3(&program.position)
            .unbind();
        let _ = ArrayBuffer::create(ctx)?
            .bind()
            .set_buffer_data(&hexatile.normals)
            .set_vertex_attribute_pointer_vec3(&program.normal)
            .unbind();
        let _ = ArrayBuffer::create(ctx)?
            .bind()
            .set_buffer_data(&hexatile.instance_model_data)
            .set_vertex_attribute_pointer_mat4(&program.model)
            .set_vertex_attrib_divisor_mat4(&program.model, 1)
            .unbind();
        let _ = ArrayBuffer::create(ctx)?
            .bind()
            .set_buffer_data(&hexatile.instance_normals_data)
            .set_vertex_attribute_pointer_mat4(&program.normals)
            .set_vertex_attrib_divisor_mat4(&program.normals, 1)
            .unbind();

        program.position.enable();
        program.normal.enable();
        program.model.enable();
        program.normals.enable();

        vao_hexatile.unbind();

        // clear

        ctx.gl.clear_color(0.7, 0.7, 0.7, 1.0);
        ctx.gl
            .clear(web_sys::WebGlRenderingContext::COLOR_BUFFER_BIT);

        // draw

        program.activate();
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

    fn render(&mut self, _ctx: &Context, millis: f64) -> Result<(), JsValue> {
        self.last_render = Duration::from_micros((millis * 1000.0) as u64);
        Ok(())
    }

    fn done(&self) -> bool {
        self.last_render >= Duration::from_secs(3)
    }
}

// TODO: use const generic for event type name.
impl engine::EventHandler<web_sys::MouseEvent> for Game {
    fn handle(&mut self, millis: f64, event: &web_sys::MouseEvent) {
        // TODO: Experiment with a #[wasm_bindgen(inline_js) function
        //       that does most calls in JS.
        let r = event
            .target()
            .unwrap()
            .unchecked_ref::<web_sys::Element>()
            .get_bounding_client_rect();
        let x = event.client_x() - r.left() as i32;
        let y = event.client_y() - r.top() as i32;
        log::debug!("Clicked at {}: {},{}", millis, x, y);
    }
}
