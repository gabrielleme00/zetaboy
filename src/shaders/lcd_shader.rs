pub const LCD_VERTEX_SHADER: &str = r#"
#version 330 core
layout(location = 0) in vec2 a_pos;
layout(location = 1) in vec2 a_uv;

out vec2 v_uv;

void main() {
    v_uv = a_uv;
    gl_Position = vec4(a_pos, 0.0, 1.0);
}
"#;

pub const LCD_FRAGMENT_SHADER: &str = r#"
#version 330 core
precision mediump float;

in vec2 v_uv;
out vec4 FragColor;

uniform sampler2D u_texture;

void main() {
    FragColor = texture(u_texture, v_uv);
}
"#;
