use ::std::{
    rc::Rc,
    result::{Result, Result::Ok},
};

use crate::{
    engine::{attrib, Error},
    util::{
        math::Mat4,
        opengl::{
            Context, Program, Shader,
            ShaderType::{Fragment, Vertex},
            Uniform,
        },
    },
};

pub struct PickerShader {
    program: Program,

    view: Uniform,
    projection: Uniform,
}

impl PickerShader {
    pub fn new(ctx: &Rc<Context>) -> Result<Self, Error> {
        let mut vertex_shader = Shader::create(ctx, Vertex)?;
        vertex_shader.compile_source(PICKER_VERTEX_SHADER)?;
        let mut fragment_shader = Shader::create(ctx, Fragment)?;
        fragment_shader.compile_source(PICKER_FRAGMENT_SHADER)?;

        let mut program = Program::create(ctx)?;
        program.attach_shader(&vertex_shader);
        program.attach_shader(&fragment_shader);

        program.bind_attrib_location(attrib::POSITION, "position");
        program.bind_attrib_location(attrib::INSTANCE_ID, "instance_id");
        program.bind_attrib_location(attrib::MODEL, "model");

        program.link()?;

        let view = Uniform::find(ctx, &program, "view")?;
        let projection = Uniform::find(ctx, &program, "projection")?;

        Ok(PickerShader {
            program,
            view,
            projection,
        })
    }

    pub fn activate(&mut self) {
        self.program.r#use();
    }

    pub fn set_view(&mut self, m: &Mat4) {
        self.view.set_mat4(m.slice());
    }

    pub fn set_projection(&mut self, m: &Mat4) {
        self.projection.set_mat4(m.slice());
    }
}

const PICKER_VERTEX_SHADER: &str = r#"#version 300 es

// per vertex
layout(location=0) in vec3 position;
// per instance
layout(location=3) in vec3 instance_id;
layout(location=4) in mat4 model;

uniform mat4 view;
uniform mat4 projection;

out highp vec4 codecolor;

void main() {
    gl_Position = projection * view * model * vec4(position, 1.0);
    codecolor = vec4(instance_id, 1.0);
}
"#;

const PICKER_FRAGMENT_SHADER: &str = r#"#version 300 es

in highp vec4 codecolor;

layout(location=0) out highp vec4 fragcolor;

void main() {
    fragcolor = codecolor;
}
"#;
