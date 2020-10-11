use ::std::rc::Rc;
use ::std::result::{Result, Result::Ok};

use crate::engine::attrib;
use crate::engine::Error;
use crate::util::math::Mat4;
use crate::util::opengl::{
    Context, Program, Shader, ShaderType::Fragment, ShaderType::Vertex, Uniform,
};

pub struct MaterialShader {
    program: Program,

    view: Uniform,
    projection: Uniform,
}

impl MaterialShader {
    pub fn new(ctx: &Rc<Context>) -> Result<Self, Error> {
        let mut vertex_shader = Shader::create(ctx, Vertex)?;
        vertex_shader.compile_source(MATERIAL_VERTEX_SHADER)?;
        let mut fragment_shader = Shader::create(ctx, Fragment)?;
        fragment_shader.compile_source(MATERIAL_FRAGMENT_SHADER)?;

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

        Ok(MaterialShader {
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

const MATERIAL_VERTEX_SHADER: &str = r#"#version 300 es

// per vertex
layout(location=0) in vec3 position;
layout(location=1) in vec3 normal;
// per instance
layout(location=2) in vec3 color;
layout(location=4) in mat4 model;
layout(location=8) in mat4 normals;

uniform mat4 view;
uniform mat4 projection;

out highp vec3 f_fragpos;
out highp vec3 f_normal;
out highp vec3 f_basecolor;
out highp vec3 f_lighting;

void main() {
    vec4 pos = model * vec4(position, 1.0);
    gl_Position = projection * view * pos;
    f_fragpos = vec3(pos);
    f_normal = vec3(normals * vec4(normal, 1.0));
    f_basecolor = color;
}
"#;

const MATERIAL_FRAGMENT_SHADER: &str = r#"#version 300 es

in highp vec3 f_fragpos;
in highp vec3 f_normal;
in highp vec3 f_basecolor;

layout(location=0) out highp vec4 fragcolor;

struct Light {
    highp vec3 position;

    highp float ambient;
    highp float diffuse;
    highp float specular;

    highp float constant;
    highp float linear;
    highp float quadratic;
};

struct Material {
    highp vec3 ambient;
    highp vec3 diffuse;
    highp vec3 specular;
    highp float shininess;
};

void main() {
    highp vec3 cameraPos = vec3(0.5, 1.0, 3.0);

    // TODO: Define uniform for light source(s).
    Light light = Light(
        vec3(1.5, 3.0, 2.0),
        0.2,
        0.5,
        1.0,
        1.0,
        0.03,
        0.002
    );

    // TODO: Define uniform for material(s).
    Material material = Material(
        f_basecolor,
        f_basecolor,
        vec3(0.5, 0.5, 0.5),
        32.0
    );

    highp vec3 ambient = vec3(light.ambient) * material.ambient;

    highp vec3 lightDir = normalize(light.position - f_fragpos);
    highp float diff = max(dot(f_normal, lightDir), 0.0);
    highp vec3 diffuse = vec3(light.diffuse) * (diff * material.diffuse);

    highp vec3 viewDir = normalize(cameraPos - f_fragpos);
    highp vec3 reflectDir = reflect(-lightDir, f_normal);
    highp float spec = pow(max(dot(viewDir, reflectDir), 0.0), material.shininess);
    highp vec3 specular = vec3(light.specular) * (spec * material.specular);

    highp float distance = length(light.position - f_fragpos);
    highp float attenuation = 1.0 / (
        light.constant +
        light.linear * distance +
        light.quadratic * (distance * distance)
    );

    fragcolor = vec4((ambient + diffuse + specular) * attenuation, 1.0);
}
"#;
