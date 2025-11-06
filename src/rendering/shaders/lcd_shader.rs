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
uniform vec2 u_resolution;

void main() {
    // Get the pixel coordinate in texture space
    vec2 pixelCoord = v_uv * u_resolution;
    vec2 pixelIndex = floor(pixelCoord);
    vec2 pixelPos = fract(pixelCoord);
    
    // Calculate distance from center of pixel (0.5, 0.5)
    vec2 center = vec2(0.5, 0.5);
    vec2 dist = abs(pixelPos - center);
    
    // Create rounded square shape for each pixel
    // Using max distance to create a diamond/rounded square appearance
    float distMax = max(dist.x, dist.y);
    float radius = 0.42;
    float softness = 0.08;
    
    // Smooth pixel shape with soft edges
    float alpha = 1.0 - smoothstep(radius - softness, radius + softness, distMax);
    
    // Sample the texture at the center of the pixel
    vec2 texCoord = (pixelIndex + 0.5) / u_resolution;
    vec4 pixelColor = texture(u_texture, texCoord);
    
    // Light grid/background color (slightly greenish tint like DMG LCD)
    vec4 gridColor = vec4(0.05, 0.08, 0.02, 0.1);
    
    // Mix between grid and pixel color based on alpha
    FragColor = mix(gridColor, pixelColor, alpha);
}
"#;
