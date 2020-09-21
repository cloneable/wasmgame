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

use std::option::{Option::None, Option::Some};
use std::rc::Rc;
use std::result::{Result, Result::Ok};
use std::time::Duration;

use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;

use engine::attrib;
use engine::math::Mat4;
use engine::opengl::{
    ArrayBuffer, Context, Framebuffer, Renderbuffer, Texture2D, VertexArrayObject,
};
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
    fn setup(&mut self, ctx: &Rc<Context>) -> Result<(), JsValue> {
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

        let mut picker_program = engine::picker::PickerProgram::new(ctx)?;
        picker_program.activate();
        picker_program.set_view(cam.view_matrix());
        picker_program.set_projection(cam.projection_matrix());

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
            .set_vertex_attribute_pointer_vec3(attrib::POSITION)
            .unbind();
        let _ = ArrayBuffer::create(ctx)?
            .bind()
            .set_buffer_data(&hexatile.normals)
            .set_vertex_attribute_pointer_vec3(attrib::NORMAL)
            .unbind();
        let _ = ArrayBuffer::create(ctx)?
            .bind()
            .set_buffer_data(&hexatile.instance_id)
            .set_vertex_attribute_pointer_vec3(attrib::INSTANCE_ID)
            .set_vertex_attrib_divisor(attrib::INSTANCE_ID, 1)
            .unbind();
        let _ = ArrayBuffer::create(ctx)?
            .bind()
            .set_buffer_data(&hexatile.instance_model_data)
            .set_vertex_attribute_pointer_mat4(attrib::MODEL)
            .set_vertex_attrib_divisor(attrib::MODEL, 1)
            .unbind();
        let _ = ArrayBuffer::create(ctx)?
            .bind()
            .set_buffer_data(&hexatile.instance_normals_data)
            .set_vertex_attribute_pointer_mat4(attrib::NORMALS)
            .set_vertex_attrib_divisor(attrib::NORMALS, 1)
            .unbind();

        attrib::POSITION.enable(ctx);
        attrib::NORMAL.enable(ctx);
        attrib::MODEL.enable(ctx);
        attrib::NORMALS.enable(ctx);
        attrib::INSTANCE_ID.enable(ctx);

        vao_hexatile.unbind();

        let mut fb_colorbuffer = Texture2D::create(ctx)?;
        fb_colorbuffer.bind();
        fb_colorbuffer.tex_image_2d(400, 300, None)?;
        fb_colorbuffer.unbind();

        let mut fb_depthbuffer = Renderbuffer::create(ctx)?;
        fb_depthbuffer.bind();
        fb_depthbuffer.storage_for_depth(400, 300);
        fb_depthbuffer.unbind();

        let mut fb = Framebuffer::create(ctx)?;
        fb.bind();
        fb.texture2d_as_colorbuffer(&fb_colorbuffer);
        fb.renderbuffer_as_depthbuffer(&fb_depthbuffer);
        {
            let fb_status = fb.check_status();
            if fb_status != web_sys::WebGlRenderingContext::FRAMEBUFFER_COMPLETE {
                log::error!("framebuffer incomplete: {}", fb_status)
            }
        }
        fb.unbind();

        // clear

        ctx.gl.clear_color(0.0, 0.0, 0.0, 0.0);
        ctx.gl
            .clear(web_sys::WebGlRenderingContext::COLOR_BUFFER_BIT);

        // draw

        vao_hexatile.bind();

        picker_program.activate();
        fb.bind();
        ctx.instanced_arrays_ext.draw_arrays_instanced_angle(
            web_sys::WebGlRenderingContext::TRIANGLES,
            0,
            hexatile.vertices.len() as i32 / 3,
            hexatile.instances.len() as i32,
        );
        fb.unbind();

        program.activate();
        ctx.instanced_arrays_ext.draw_arrays_instanced_angle(
            web_sys::WebGlRenderingContext::TRIANGLES,
            0,
            hexatile.vertices.len() as i32 / 3,
            hexatile.instances.len() as i32,
        );

        vao_hexatile.unbind();

        // TODO: for read_pixels.
        //fb.bind();

        Ok(())
    }

    fn render(&mut self, _ctx: &Rc<Context>, millis: f64) -> Result<(), JsValue> {
        self.last_render = Duration::from_micros((millis * 1000.0) as u64);
        Ok(())
    }

    fn done(&self) -> bool {
        self.last_render >= Duration::from_secs(3)
    }
}

// TODO: use const generic for event type name.
impl engine::EventHandler<web_sys::MouseEvent> for Game {
    fn handle(&mut self, ctx: &Context, millis: f64, event: &web_sys::MouseEvent) {
        // TODO: Experiment with a #[wasm_bindgen(inline_js) function
        //       that does most calls in JS.
        let r = event
            .target()
            .unwrap()
            .unchecked_ref::<web_sys::Element>()
            .get_bounding_client_rect();
        let x = event.client_x() - r.left() as i32;
        let y = event.client_y() - r.top() as i32;
        let rgba: &mut [u8] = &mut [0, 0, 0, 0];
        ctx.gl
            .read_pixels_with_opt_u8_array(
                x,
                r.height() as i32 - y,
                1,
                1,
                web_sys::WebGlRenderingContext::RGBA,
                web_sys::WebGlRenderingContext::UNSIGNED_BYTE,
                Some(rgba),
            )
            .unwrap();
        log::debug!(
            "Clicked at {}: {},{}; rgba = {} {} {} {}",
            millis,
            x,
            y,
            rgba[0],
            rgba[1],
            rgba[2],
            rgba[3]
        );
    }
}
