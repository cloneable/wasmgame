pub const HEXATILE_VERTEX_SHADER: &str = r#"
#version 100

attribute vec3 position;
attribute vec3 normal;

uniform mat4 mvp;
uniform mat4 normals;

varying vec3 lighting;

void main() {
    gl_Position = mvp * vec4(position, 1.0);

    vec3 ambientLightColor = vec3(0.2, 0.2, 0.2);
    vec3 directionalLightColor = vec3(1.0, 1.0, 1.0);
    vec4 directionalLight = vec4(-2.0, -6.0, 2.0, 0.0);

    float intensity = max(dot(normals * vec4(normal, 0.0), normalize(directionalLight)), 0.0);
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
