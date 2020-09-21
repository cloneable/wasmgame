extern crate js_sys;
extern crate log;
extern crate std;
extern crate wasm_bindgen;
extern crate wasm_bindgen_macro;
extern crate wasm_logger;
extern crate web_sys;

use crate::engine;

use std::rc::Rc;
use std::result::{Result, Result::Ok};

use wasm_bindgen::JsValue;

use engine::attrib;
use engine::math::{Mat4, Vec4};
use engine::opengl::{ArrayBuffer, Context, VertexArrayObject};
use engine::scene::Model;

pub struct Hexatile {
    pub model: Model,

    vao: VertexArrayObject,

    vbo_vertex: ArrayBuffer,
    vbo_normals: ArrayBuffer,
    vbo_instance_color: ArrayBuffer,
    vbo_instance_id: ArrayBuffer,
    vbo_instance_models: ArrayBuffer,
    vbo_instance_normals: ArrayBuffer,
}

impl Hexatile {
    pub fn new(ctx: &Rc<Context>) -> Result<Self, JsValue> {
        Ok(Hexatile {
            model: Model::new(
                &super::meshes::HEXATILE_VERTICES,
                &super::meshes::HEXATILE_INDICES,
            ),
            vao: VertexArrayObject::create(ctx)?,
            vbo_vertex: ArrayBuffer::create(ctx)?,
            vbo_normals: ArrayBuffer::create(ctx)?,
            vbo_instance_color: ArrayBuffer::create(ctx)?,
            vbo_instance_id: ArrayBuffer::create(ctx)?,
            vbo_instance_models: ArrayBuffer::create(ctx)?,
            vbo_instance_normals: ArrayBuffer::create(ctx)?,
        })
    }
}

impl engine::scene::Drawable for Hexatile {
    fn init(&mut self, ctx: &Rc<Context>, camera: &engine::scene::Camera) {
        // lightpink: #ffb6c1
        // lightskyblue: #87cefa
        // midnightblue: #191970
        self.model.add_instance(
            Mat4::with_array([
                1.0, 0.0, 0.0, 0.0, //br
                0.0, 3.0, 0.0, 0.0, //br
                0.0, 0.0, 1.0, 0.0, //br
                -0.6, 0.0, 0.0, 1.0, //br
            ]),
            Vec4::rgb(0x19, 0x19, 0x70),
        );
        self.model.add_instance(
            Mat4::with_array([
                1.0, 0.0, 0.0, 0.0, //br
                0.0, 2.0, 0.0, 0.0, //br
                0.0, 0.0, 1.0, 0.0, //br
                0.0, 0.0, 0.0, 1.0, //br
            ]),
            Vec4::rgb(0x87, 0xce, 0xfa),
        );
        self.model.add_instance(
            Mat4::with_array([
                1.0, 0.0, 0.0, 0.0, //br
                0.0, 1.0, 0.0, 0.0, //br
                0.0, 0.0, 1.0, 0.0, //br
                0.6, 0.0, 0.0, 1.0, //br
            ]),
            Vec4::rgb(0xff, 0xb6, 0xc1),
        );
        self.model.update_instances(camera);

        self.vao.bind();
        self.vbo_vertex
            .bind()
            .set_buffer_data(&self.model.vertices)
            .set_vertex_attribute_pointer_vec3(attrib::POSITION)
            .unbind();
        self.vbo_normals
            .bind()
            .set_buffer_data(&self.model.normals)
            .set_vertex_attribute_pointer_vec3(attrib::NORMAL)
            .unbind();
        self.vbo_instance_color
            .bind()
            .set_buffer_data(&self.model.instance_color)
            .set_vertex_attribute_pointer_vec3(attrib::INSTANCE_COLOR)
            .set_vertex_attrib_divisor(attrib::INSTANCE_COLOR, 1)
            .unbind();
        self.vbo_instance_id
            .bind()
            .set_buffer_data(&self.model.instance_id)
            .set_vertex_attribute_pointer_vec3(attrib::INSTANCE_ID)
            .set_vertex_attrib_divisor(attrib::INSTANCE_ID, 1)
            .unbind();
        self.vbo_instance_models
            .bind()
            .set_buffer_data(&self.model.instance_model_data)
            .set_vertex_attribute_pointer_mat4(attrib::MODEL)
            .set_vertex_attrib_divisor(attrib::MODEL, 1)
            .unbind();
        self.vbo_instance_normals
            .bind()
            .set_buffer_data(&self.model.instance_normals_data)
            .set_vertex_attribute_pointer_mat4(attrib::NORMALS)
            .set_vertex_attrib_divisor(attrib::NORMALS, 1)
            .unbind();

        attrib::POSITION.enable(ctx);
        attrib::NORMAL.enable(ctx);
        attrib::INSTANCE_COLOR.enable(ctx);
        attrib::INSTANCE_ID.enable(ctx);
        attrib::MODEL.enable(ctx);
        attrib::NORMALS.enable(ctx);

        self.vao.unbind();
    }

    fn update(&mut self, _ctx: &Rc<Context>, _camera: &engine::scene::Camera) {}

    fn stage(&mut self, _ctx: &Rc<Context>) {
        self.vao.bind();
    }

    fn draw(&self, ctx: &Rc<Context>) {
        ctx.instanced_arrays_ext.draw_arrays_instanced_angle(
            web_sys::WebGlRenderingContext::TRIANGLES,
            0,
            self.model.vertices.len() as i32 / 3,
            self.model.instances.len() as i32,
        );
    }

    fn unstage(&mut self, _ctx: &Rc<Context>) {
        self.vao.unbind();
    }
}
