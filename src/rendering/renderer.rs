use eframe::egui_glow::glow;

pub struct GlContext {
    pub program: glow::Program,
    pub vao: glow::VertexArray,
    pub vbo: glow::Buffer,
    pub ebo: glow::Buffer,
    pub texture: glow::Texture,
}

pub fn render_with_shader(gl: &glow::Context, ctx: &GlContext, image_buffer: &[u32]) {
    use crate::emulator::ppu::{HEIGHT, WIDTH};
    use eframe::egui_glow::glow::HasContext;

    unsafe {
        // Update current frame texture
        gl.active_texture(glow::TEXTURE0);
        gl.bind_texture(glow::TEXTURE_2D, Some(ctx.texture));

        // Convert buffer to RGBA format
        let mut rgba_data = Vec::with_capacity(WIDTH * HEIGHT * 4);
        for &pixel in image_buffer {
            rgba_data.push((pixel >> 16) as u8); // R
            rgba_data.push((pixel >> 8) as u8); // G
            rgba_data.push(pixel as u8); // B
            rgba_data.push(255); // A
        }

        // Upload texture data
        gl.tex_image_2d(
            glow::TEXTURE_2D,
            0,
            glow::RGBA as i32,
            WIDTH as i32,
            HEIGHT as i32,
            0,
            glow::RGBA,
            glow::UNSIGNED_BYTE,
            glow::PixelUnpackData::Slice(Some(&rgba_data)),
        );

        // Use shader program
        gl.use_program(Some(ctx.program));

        // Set texture uniform (texture is bound to unit 0)
        if let Some(location) = gl.get_uniform_location(ctx.program, "u_texture") {
            gl.uniform_1_i32(Some(&location), 0);
        }

        // Set resolution uniform
        if let Some(location) = gl.get_uniform_location(ctx.program, "u_resolution") {
            gl.uniform_2_f32(Some(&location), WIDTH as f32, HEIGHT as f32);
        }

        // Draw the quad
        gl.bind_vertex_array(Some(ctx.vao));
        gl.draw_elements(glow::TRIANGLES, 6, glow::UNSIGNED_INT, 0);

        // Cleanup
        gl.bind_vertex_array(None);
        gl.use_program(None);
    }
}
