use ::std::clone::Clone;
use ::std::option::{Option::None, Option::Some};
use ::std::rc::Rc;
use ::std::result::{Result, Result::Ok};
use ::std::{assert, panic};
use ::std::{vec, vec::Vec};

use super::attrib;
use super::util;
use crate::engine::Error;
use crate::util::math::{look_at, project, Mat4, Vec3, Vec4};
use crate::util::opengl::{ArrayBuffer, Context, VertexArrayObject};

pub trait Drawable {
    fn init(&mut self, camera: &Camera);
    fn update(&mut self, camera: &Camera);
    fn stage(&mut self);
    fn draw(&self);
    fn unstage(&mut self);
}

pub struct Camera {
    position: Vec3,
    target: Vec3,
    up: Vec3,

    fov: f32,
    aspect: f32,
    near: f32,
    far: f32,

    view: Mat4,
    projection: Mat4,
}

impl Camera {
    pub fn new() -> Self {
        Camera {
            position: Vec3::with(0.0, 0.0, 1.0),
            target: Vec3::with(0.0, 0.0, 0.0),
            up: Vec3::with(0.0, 1.0, 0.0),
            fov: 90.0,
            aspect: 1.0,
            near: 0.1,
            far: 1000.0,
            view: Mat4::identity(),
            projection: Mat4::identity(),
        }
    }

    pub fn position(&self) -> Vec3 {
        self.position
    }

    pub fn set_position(&mut self, x: f32, y: f32, z: f32) -> &mut Self {
        self.position = Vec3::with(x, y, z);
        self
    }

    pub fn set_target(&mut self, x: f32, y: f32, z: f32) -> &mut Self {
        self.target = Vec3::with(x, y, z);
        self
    }

    pub fn set_frustum(
        &mut self, fov: f32, aspect: f32, near: f32, far: f32,
    ) -> &mut Self {
        self.fov = fov;
        self.aspect = aspect;
        self.near = near;
        self.far = far;
        self
    }

    pub fn refresh(&mut self) -> &mut Self {
        self.view = look_at(self.position, self.target, self.up);
        self.projection = project(self.fov, self.aspect, self.near, self.far);
        self
    }

    pub fn view_matrix(&self) -> &Mat4 {
        &self.view
    }

    pub fn projection_matrix(&self) -> &Mat4 {
        &self.projection
    }
}

pub struct Model {
    ctx: Rc<Context>,

    vertices: Vec<f32>,
    normals: Vec<f32>,

    instances: Vec<Instance>,

    vao: VertexArrayObject,

    // static, per vertex
    vbo_vertex: ArrayBuffer,
    vbo_normals: ArrayBuffer,
    // static, per instance
    vbo_instance_color: ArrayBuffer,
    vbo_instance_id: ArrayBuffer,
    // dynamic, per instance
    vbo_instance_models: ArrayBuffer,
    vbo_instance_normals: ArrayBuffer,

    instance_color: Vec<f32>,
    instance_id: Vec<f32>,
    instance_model_data: Vec<f32>,
    instance_normals_data: Vec<f32>,
}

impl Model {
    pub fn new(
        ctx: &Rc<Context>, indexed_vertices: &'static [f32],
        indices: &'static [u8], num_instances: usize,
    ) -> Result<Self, Error> {
        assert!(num_instances > 0);
        let mut vertices: Vec<f32> = vec![0.0; indices.len() * 3];
        let mut normals: Vec<f32> = vec![0.0; indices.len() * 3];
        util::generate_buffers(
            indices,
            indexed_vertices,
            &mut vertices,
            &mut normals,
        );

        let mut instances: Vec<Instance> = Vec::with_capacity(num_instances);
        instances.resize_with(num_instances, Instance::new);

        Ok(Model {
            ctx: ctx.clone(),
            vertices,
            normals,
            instances,
            vao: VertexArrayObject::create(ctx)?,
            vbo_vertex: ArrayBuffer::create(ctx)?,
            vbo_normals: ArrayBuffer::create(ctx)?,
            vbo_instance_color: ArrayBuffer::create(ctx)?,
            vbo_instance_id: ArrayBuffer::create(ctx)?,
            vbo_instance_models: ArrayBuffer::create(ctx)?,
            vbo_instance_normals: ArrayBuffer::create(ctx)?,
            instance_color: Vec::<f32>::new(),
            instance_id: Vec::<f32>::new(),
            instance_model_data: Vec::<f32>::new(),
            instance_normals_data: Vec::<f32>::new(),
        })
    }

    pub fn init(&mut self) {
        let mut i: i32 = 1;
        for instance in self.instances.iter_mut() {
            self.instance_color.push(instance.color.x);
            self.instance_color.push(instance.color.y);
            self.instance_color.push(instance.color.z);
            self.instance_id.push(i as f32 / 255.0);
            self.instance_id.push(1.0);
            self.instance_id.push(1.0);
            self.instance_model_data
                .extend_from_slice(instance.model.slice());
            self.instance_normals_data
                .extend_from_slice(instance.normals.slice());
            i += 1;
        }

        self.vao.bind();

        self.vbo_vertex
            .bind()
            .set_buffer_data(&self.vertices)
            .set_vertex_attribute_pointer_vec3(attrib::POSITION)
            .unbind();
        self.vbo_normals
            .bind()
            .set_buffer_data(&self.normals)
            .set_vertex_attribute_pointer_vec3(attrib::NORMAL)
            .unbind();
        self.vbo_instance_color
            .bind()
            .set_buffer_data(&self.instance_color)
            .set_vertex_attribute_pointer_vec3(attrib::INSTANCE_COLOR)
            .set_vertex_attrib_divisor(attrib::INSTANCE_COLOR, 1)
            .unbind();
        self.vbo_instance_id
            .bind()
            .set_buffer_data(&self.instance_id)
            .set_vertex_attribute_pointer_vec3(attrib::INSTANCE_ID)
            .set_vertex_attrib_divisor(attrib::INSTANCE_ID, 1)
            .unbind();
        self.vbo_instance_models
            .bind()
            .allocate_dynamic(16 * 4 * self.instances.len())
            .set_vertex_attribute_pointer_mat4(attrib::MODEL)
            .set_vertex_attrib_divisor(attrib::MODEL, 1)
            .unbind();
        self.vbo_instance_normals
            .bind()
            .allocate_dynamic(16 * 4 * self.instances.len())
            .set_vertex_attribute_pointer_mat4(attrib::NORMALS)
            .set_vertex_attrib_divisor(attrib::NORMALS, 1)
            .unbind();

        attrib::POSITION.enable(&self.ctx);
        attrib::NORMAL.enable(&self.ctx);
        attrib::INSTANCE_COLOR.enable(&self.ctx);
        attrib::INSTANCE_ID.enable(&self.ctx);
        attrib::MODEL.enable(&self.ctx);
        attrib::NORMALS.enable(&self.ctx);

        self.vao.unbind();
    }

    pub fn refresh(&mut self) {
        self.instance_model_data.clear();
        self.instance_normals_data.clear();
        for instance in self.instances.iter_mut() {
            self.instance_model_data
                .extend_from_slice(instance.model.slice());
            self.instance_normals_data
                .extend_from_slice(instance.normals.slice());
        }
        self.vbo_instance_models.bind();
        self.vbo_instance_models
            .set_buffer_sub_data(0, &self.instance_model_data);
        self.vbo_instance_normals.bind();
        self.vbo_instance_normals
            .set_buffer_sub_data(0, &self.instance_normals_data);
        self.vbo_instance_normals.unbind();
    }

    pub fn select(&mut self) {
        self.vao.bind();
    }

    pub fn draw(&self) {
        self.ctx.instanced_arrays_ext.draw_arrays_instanced_angle(
            ::web_sys::WebGlRenderingContext::TRIANGLES,
            0,
            self.vertices.len() as i32 / 3,
            self.instances.len() as i32,
        );
    }

    pub fn unselect(&mut self) {
        self.vao.unbind();
    }
}

impl ::std::ops::Index<usize> for Model {
    type Output = Instance;
    fn index(&self, i: usize) -> &Instance {
        &self.instances[i]
    }
}

impl ::std::ops::IndexMut<usize> for Model {
    fn index_mut(&mut self, i: usize) -> &mut Instance {
        &mut self.instances[i]
    }
}

pub struct Instance {
    pub position: Vec3,
    pub scale: Vec3,
    pub rotation: Vec3,
    pub color: Vec4,

    pub model: Mat4,
    pub normals: Mat4,
}

impl Instance {
    pub fn new() -> Self {
        Instance {
            position: Vec3::with(0.0, 0.0, 0.0),
            scale: Vec3::with(1.0, 1.0, 1.0),
            rotation: Vec3::with(0.0, 0.0, 0.0),
            color: Vec4::with(1.0, 1.0, 1.0, 1.0),
            model: Mat4::identity(),
            normals: Mat4::identity(),
        }
    }

    pub fn translate(&mut self, v: Vec3) {
        self.position += v;
    }

    pub fn scale(&mut self, scalars: Vec3) {
        // TODO: Add Hadamard product to Vec34?
        self.scale.x *= scalars.x;
        self.scale.y *= scalars.y;
        self.scale.z *= scalars.z;
    }

    pub fn rotate(&mut self, angles: Vec3) {
        self.rotation += angles;
    }

    pub fn color(&mut self, rgba: Vec4) {
        self.color = rgba;
    }

    pub fn refresh(&mut self, view: &Mat4) {
        let t = Mat4::translation(self.position);
        let s = Mat4::scaling(self.scale);
        let r = Mat4::rotation(self.rotation);
        let m = t * r * s;
        let mat_model_view = (view * &m).to_3x3();
        let normals = match mat_model_view.invert() {
            Some(inv) => inv.transpose(),
            None => mat_model_view,
        };
        self.model = m;
        self.normals = normals;
    }
}
