extern crate std;
extern crate wasm_bindgen;

use std::rc::Rc;
use std::result::{Result, Result::Ok};

use wasm_bindgen::JsValue;

use crate::engine::attrib;
use crate::engine::math::Mat4;
use crate::engine::opengl::{
    Context, Program, Shader, ShaderType::Fragment, ShaderType::Vertex, Uniform,
};

pub struct HexatileProgram {
    program: Program,

    view: Uniform,
    projection: Uniform,
}

impl HexatileProgram {
    pub fn new(ctx: &Rc<Context>) -> Result<Self, JsValue> {
        let mut vertex_shader = Shader::create(ctx, Vertex)?;
        vertex_shader.compile_source(HEXATILE_VERTEX_SHADER)?;
        let mut fragment_shader = Shader::create(ctx, Fragment)?;
        fragment_shader.compile_source(HEXATILE_FRAGMENT_SHADER)?;

        let mut program = Program::create(ctx)?;
        program.attach_shader(&vertex_shader);
        program.attach_shader(&fragment_shader);

        attrib::POSITION.bind(ctx, &program, "position");
        attrib::NORMAL.bind(ctx, &program, "normal");
        attrib::MODEL.bind(ctx, &program, "model");
        attrib::NORMALS.bind(ctx, &program, "normals");

        program.link()?;

        let view = Uniform::find(ctx, &program, "view")?;
        let projection = Uniform::find(ctx, &program, "projection")?;

        Ok(HexatileProgram {
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

const HEXATILE_VERTEX_SHADER: &str = r#"
#version 100

// per vertex
attribute vec3 position;
attribute vec3 normal;
// per instance
attribute mat4 model;
attribute mat4 normals;

uniform mat4 view;
uniform mat4 projection;

varying highp vec3 lighting;

void main() {
    gl_Position = projection * view * model * vec4(position, 1.0);

    // TODO: define uniforms for these.
    highp vec3 ambientLightColor = vec3(0.1, 0.1, 0.2);
    highp vec3 directionalLightColor = vec3(0.9, 0.9, 0.8);
    highp vec3 directionalLight = normalize(vec3(3.0, 4.0, 5.0));

    highp vec4 transformedNormal = normalize(normals * vec4(normal, 1.0));
    highp float intensity = max(dot(transformedNormal.xyz, directionalLight), 0.0);
    lighting = ambientLightColor + (directionalLightColor * intensity);
}
"#;

const HEXATILE_FRAGMENT_SHADER: &str = r#"
#version 100

varying highp vec3 lighting;

void main() {
    highp vec4 baseColor = vec4(0.2, 0.7, 0.1, 1.0);
    gl_FragColor = vec4(baseColor.rgb * lighting, baseColor.a);
}
"#;
