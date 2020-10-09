use ::std::rc::Rc;
use ::std::result::{Result, Result::Ok};

use crate::engine::attrib;
use crate::engine::Error;
use crate::util::math::Mat4;
use crate::util::opengl::{
    Context, Program, Shader, ShaderType::Fragment, ShaderType::Vertex, Uniform,
};

pub struct HexatileProgram {
    program: Program,

    view: Uniform,
    projection: Uniform,
}

impl HexatileProgram {
    pub fn new(ctx: &Rc<Context>) -> Result<Self, Error> {
        let mut vertex_shader = Shader::create(ctx, Vertex)?;
        vertex_shader.compile_source(HEXATILE_VERTEX_SHADER)?;
        let mut fragment_shader = Shader::create(ctx, Fragment)?;
        fragment_shader.compile_source(HEXATILE_FRAGMENT_SHADER)?;

        let mut program = Program::create(ctx)?;
        program.attach_shader(&vertex_shader);
        program.attach_shader(&fragment_shader);

        attrib::POSITION.bind(ctx, &program, "position");
        attrib::NORMAL.bind(ctx, &program, "normal");
        attrib::INSTANCE_COLOR.bind(ctx, &program, "color");
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

const HEXATILE_VERTEX_SHADER: &str = r#"#version 300 es

// per vertex
layout(location=0) in vec3 position;
layout(location=1) in vec3 normal;
// per instance
layout(location=2) in vec3 color;
layout(location=4) in mat4 model;
layout(location=8) in mat4 normals;

uniform mat4 view;
uniform mat4 projection;

// TODO: combine these two.
out highp vec3 basecolor;
out highp vec3 lighting;

void main() {
    gl_Position = projection * view * model * vec4(position, 1.0);
    basecolor = color;

    // TODO: define uniforms for these.
    highp vec3 ambientLightColor = vec3(0.1, 0.1, 0.1);
    highp vec3 directionalLightColor = vec3(1.0, 1.0, 1.0);
    highp vec3 directionalLight = normalize(vec3(3.0, 3.0, 5.0));

    highp vec4 transformedNormal = normalize(normals * vec4(normal, 1.0));
    highp float intensity = max(dot(transformedNormal.xyz, directionalLight), 0.0);
    lighting = ambientLightColor + (directionalLightColor * intensity);
}
"#;

const HEXATILE_FRAGMENT_SHADER: &str = r#"#version 300 es

in highp vec3 basecolor;
in highp vec3 lighting;

layout(location=0) out highp vec4 fragcolor;

void main() {
    fragcolor = vec4(basecolor * lighting, 1.0);
}
"#;
