use ::std::{
    assert,
    clone::Clone,
    debug_assert,
    option::Option::{None, Some},
    panic,
    rc::Rc,
    result::{Result, Result::Ok},
    vec::Vec,
};

use crate::{
    engine::{attrib, ecs, time::Time, Error},
    util::{
        math::{look_at, project, Mat4, Vec3, Vec4},
        opengl::{ArrayBuffer, Context, VertexArrayObject},
    },
};

#[derive(Copy, Clone)]
pub struct MeshData {
    pub name: &'static str,
    pub buf: &'static [f32],
    pub start: usize,
    pub end: usize,
}

impl MeshData {
    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn data(&self) -> &'static [f32] {
        &self.buf[self.start..self.end]
    }
}

pub struct Object {
    position: Vec3,
    scaling: Vec3,
    rotation: Vec3,

    model: Mat4,
    model_stale: bool,
}

impl Object {
    pub fn new() -> Self {
        Object {
            position: Vec3::new(),
            scaling: Vec3::with(1.0, 1.0, 1.0),
            rotation: Vec3::new(),
            model: Mat4::identity(),
            model_stale: false,
        }
    }

    pub fn position(&self) -> Vec3 {
        self.position
    }

    pub fn set_position(&mut self, v: Vec3) {
        self.position = v;
        self.model_stale = true;
    }

    pub fn translate(&mut self, v: Vec3) {
        self.position += v;
        self.model_stale = true;
    }

    pub fn scaling(&self) -> Vec3 {
        self.scaling
    }

    pub fn set_scaling(&mut self, v: Vec3) {
        self.scaling = v;
        self.model_stale = true;
    }

    pub fn scale_uniform(&mut self, s: f32) {
        self.scaling *= s;
        self.model_stale = true;
    }

    pub fn scale(&mut self, v: Vec3) {
        self.scaling *= v;
        self.model_stale = true;
    }

    pub fn rotation(&self) -> Vec3 {
        self.rotation
    }

    pub fn set_rotation(&mut self, v: Vec3) {
        self.rotation = v;
        self.model_stale = true;
    }

    pub fn rotate(&mut self, v: Vec3) {
        self.rotation += v;
        self.model_stale = true;
    }

    pub fn update(&mut self, _t: Time) -> bool {
        if self.model_stale {
            let s = Mat4::scaling(self.scaling);
            let r = Mat4::rotation(self.rotation);
            let t = Mat4::translation(self.position);
            self.model = t * r * s;
            self.model_stale = false;
            return true;
        }
        return false;
    }
}

pub struct Camera {
    position: Vec3,
    target: Vec3,

    up: Vec3,
    rotation: Vec3,

    fov: f32,
    aspect: f32,
    near: f32,
    far: f32,

    view: Mat4,
    view_stale: bool,
    projection: Mat4,
    projection_stale: bool,
}

impl Camera {
    pub fn new() -> Self {
        Camera {
            position: Vec3::with(0.0, 0.0, 1.0),
            target: Vec3::new(),
            up: Vec3::with(0.0, 1.0, 0.0),
            rotation: Vec3::new(),
            fov: 90.0,
            aspect: 1.0,
            near: 0.1,
            far: 1000.0,
            view: Mat4::identity(),
            view_stale: true,
            projection: Mat4::identity(),
            projection_stale: true,
        }
    }

    pub fn set_rotation(&mut self, x: f32, y: f32) -> &mut Self {
        self.rotation.x = f32::min(f32::max(x, -180.0), 180.0);
        self.rotation.y = f32::min(f32::max(y, -90.0), 90.0);
        self.rotation.z = 0.0;
        self.view_stale = true;
        self
    }

    pub fn position(&self) -> Vec3 {
        self.position
    }

    pub fn set_position(&mut self, x: f32, y: f32, z: f32) -> &mut Self {
        self.position = Vec3::with(x, y, z);
        self.view_stale = true;
        self
    }

    pub fn set_target(&mut self, x: f32, y: f32, z: f32) -> &mut Self {
        self.target = Vec3::with(x, y, z);
        self.view_stale = true;
        self
    }

    pub fn set_frustum(
        &mut self, fov: f32, aspect: f32, near: f32, far: f32,
    ) -> &mut Self {
        self.fov = fov;
        self.aspect = aspect;
        self.near = near;
        self.far = far;
        self.projection_stale = true;
        self
    }

    pub fn update(&mut self, _t: Time) -> bool {
        let mut changed = false;
        if self.view_stale {
            // TODO: Fix orbital rotation.
            // let m = Mat4::rotation(self.rotation);
            // let position =
            //     (m * Vec4::from_vec3(self.target - self.position, 1.0)).xyz();
            let position = self.position;
            self.view = look_at(position, self.target, self.up);
            self.view_stale = false;
            changed = true;
        }
        if self.projection_stale {
            self.projection =
                project(self.fov, self.aspect, self.near, self.far);
            self.projection_stale = false;
            changed = true;
        }
        changed
    }

    pub fn view_matrix(&self) -> &Mat4 {
        debug_assert!(!self.view_stale);
        &self.view
    }

    pub fn projection_matrix(&self) -> &Mat4 {
        debug_assert!(!self.projection_stale);
        &self.projection
    }
}

impl ecs::Component for Camera {
    type Container = ecs::Singleton<Self>;
}

pub struct Mesh {
    ctx: Rc<Context>,

    mesh_data: MeshData,

    instances: Vec<Instance>,

    vao: VertexArrayObject,

    // static, per vertex
    vbo_vertex: ArrayBuffer,
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

impl Mesh {
    pub fn new(
        ctx: &Rc<Context>, mesh_data: MeshData, num_instances: usize,
    ) -> Result<Self, Error> {
        assert!(num_instances > 0);

        let mut instances: Vec<Instance> = Vec::with_capacity(num_instances);
        instances.resize_with(num_instances, Instance::new);

        Ok(Mesh {
            ctx: ctx.clone(),
            mesh_data,
            instances,
            vao: VertexArrayObject::create(ctx)?,
            vbo_vertex: ArrayBuffer::create(ctx)?,
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
}

impl crate::engine::Drawable for Mesh {
    fn init(&mut self) -> Result<(), Error> {
        let mut i: i32 = 1;
        for instance in self.instances.iter_mut() {
            self.instance_color.push(instance.color.x);
            self.instance_color.push(instance.color.y);
            self.instance_color.push(instance.color.z);
            self.instance_id.push(i as f32 / 255.0);
            self.instance_id.push(1.0);
            self.instance_id.push(1.0);
            self.instance_model_data
                .extend_from_slice(instance.object.model.slice());
            self.instance_normals_data
                .extend_from_slice(instance.normals.slice());
            i += 1;
        }

        self.vao.bind();

        self.vbo_vertex
            .bind()
            .set_buffer_data(&self.mesh_data.data())
            .set_vertex_attribute_pointer_vec3(attrib::POSITION, 6, 0)
            .set_vertex_attribute_pointer_vec3(attrib::NORMAL, 6, 3)
            .unbind();
        self.vbo_instance_color
            .bind()
            .set_buffer_data(&self.instance_color)
            .set_vertex_attribute_pointer_vec3(attrib::INSTANCE_COLOR, 3, 0)
            .set_vertex_attrib_divisor(attrib::INSTANCE_COLOR, 1)
            .unbind();
        self.vbo_instance_id
            .bind()
            .set_buffer_data(&self.instance_id)
            .set_vertex_attribute_pointer_vec3(attrib::INSTANCE_ID, 3, 0)
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
        Ok(())
    }

    fn update(&mut self, t: Time) -> Result<(), Error> {
        self.instance_model_data.clear();
        self.instance_normals_data.clear();
        for instance in self.instances.iter_mut() {
            instance.update(t);
            self.instance_model_data
                .extend_from_slice(instance.object.model.slice());
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
        Ok(())
    }

    fn draw(&mut self) -> Result<(), Error> {
        self.ctx.gl.draw_arrays_instanced(
            ::web_sys::WebGl2RenderingContext::TRIANGLES,
            0,
            self.mesh_data.data().len() as i32 / (3 + 3),
            self.instances.len() as i32,
        );
        Ok(())
    }
}

impl crate::engine::Bindable for Mesh {
    fn bind(&mut self) {
        self.vao.bind();
    }

    fn unbind(&mut self) {
        self.vao.unbind();
    }
}

impl ::std::ops::Index<usize> for Mesh {
    type Output = Instance;
    fn index(&self, i: usize) -> &Instance {
        &self.instances[i]
    }
}

impl ::std::ops::IndexMut<usize> for Mesh {
    fn index_mut(&mut self, i: usize) -> &mut Instance {
        &mut self.instances[i]
    }
}

pub struct Instance {
    pub object: Object,
    pub color: Vec4,
    pub normals: Mat4,
}

impl Instance {
    pub fn new() -> Self {
        Instance {
            object: Object::new(),
            color: Vec4::with(1.0, 1.0, 1.0, 1.0),
            normals: Mat4::identity(),
        }
    }

    pub fn color(&mut self, rgba: Vec4) {
        self.color = rgba;
    }

    pub fn update(&mut self, t: Time) -> bool {
        if self.object.update(t) {
            self.normals = {
                let m = self.object.model.to_3x3();
                match m.invert() {
                    Some(mut inv) => {
                        inv.transpose();
                        inv
                    }
                    None => m,
                }
            };
            return true;
        }
        return false;
    }
}
