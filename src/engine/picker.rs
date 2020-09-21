extern crate std;
extern crate wasm_bindgen;

use std::rc::Rc;
use std::result::{Result, Result::Ok};

use wasm_bindgen::JsValue;

use super::math::Mat4;
use super::opengl::{
    Attribute, Context, Program, Shader, ShaderType::Fragment, ShaderType::Vertex, Uniform,
};

pub struct PickerProgram {
    program: Program,

    view: Uniform,
    projection: Uniform,

    pub position: Attribute,
    pub instance_id: Attribute,
    pub model: Attribute,
}

impl PickerProgram {
    pub fn new(ctx: &Rc<Context>) -> Result<Self, JsValue> {
        let mut vertex_shader = Shader::create(ctx, Vertex)?;
        vertex_shader.compile_source(PICKER_VERTEX_SHADER)?;
        let mut fragment_shader = Shader::create(ctx, Fragment)?;
        fragment_shader.compile_source(PICKER_FRAGMENT_SHADER)?;

        let mut program = Program::create(ctx)?;
        program.attach_shader(&vertex_shader);
        program.attach_shader(&fragment_shader);

        let position = Attribute::bind(ctx, &program, 0, "position", 1)?;
        let instance_id = Attribute::bind(ctx, &program, 3, "instance_id", 1)?;
        let model = Attribute::bind(ctx, &program, 4, "model", 4)?;

        program.link()?;

        let view = Uniform::find(ctx, &program, "view")?;
        let projection = Uniform::find(ctx, &program, "projection")?;

        Ok(PickerProgram {
            program,
            view,
            projection,
            position,
            instance_id,
            model,
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

const PICKER_VERTEX_SHADER: &str = r#"
#version 100

// per vertex
attribute vec3 position;
// per instance
attribute highp vec3 instance_id;
attribute mat4 model;

uniform mat4 view;
uniform mat4 projection;

varying highp vec4 codecolor;

void main() {
    gl_Position = projection * view * model * vec4(position, 1.0);
    codecolor = vec4(instance_id, 1.0);
}
"#;

const PICKER_FRAGMENT_SHADER: &str = r#"
#version 100

varying highp vec4 codecolor;

void main() {
    gl_FragColor = codecolor;
}
"#;
