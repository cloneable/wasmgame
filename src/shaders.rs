pub const HEXATILE_VERTEX_SHADER: &str = r#"
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
    highp vec3 ambientLightColor = vec3(0.2, 0.2, 0.2);
    highp vec3 directionalLightColor = vec3(1.0, 1.0, 1.0);
    highp vec3 directionalLight = normalize(vec3(3.0, 4.0, 5.0));

    highp vec4 transformedNormal = normalize(normals * vec4(normal, 1.0));
    highp float intensity = max(dot(transformedNormal.xyz, directionalLight), 0.0);
    lighting = ambientLightColor + (directionalLightColor * intensity);
}
"#;

pub const HEXATILE_FRAGMENT_SHADER: &str = r#"
#version 100

varying highp vec3 lighting;

void main() {
    highp vec4 baseColor = vec4(1.0, 0.0, 0.0, 1.0);
    gl_FragColor = vec4(baseColor.rgb * lighting, baseColor.a);
}
"#;
