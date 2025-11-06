use crate::rendering::GlContext;
use crate::shaders::lcd_shader::{LCD_FRAGMENT_SHADER, LCD_VERTEX_SHADER};
use eframe::egui_glow::glow;

pub fn init_gl_context(gl: &glow::Context) -> Result<GlContext, String> {
    use eframe::egui_glow::glow::HasContext;

    unsafe {
        // Create shader program
        let program = gl
            .create_program()
            .map_err(|e| format!("Failed to create program: {}", e))?;

        // Compile vertex shader
        let vertex_shader = gl.create_shader(glow::VERTEX_SHADER).map_err(|e| {
            gl.delete_program(program);
            format!("Failed to create vertex shader: {}", e)
        })?;
        gl.shader_source(vertex_shader, LCD_VERTEX_SHADER);
        gl.compile_shader(vertex_shader);

        if !gl.get_shader_compile_status(vertex_shader) {
            let log = gl.get_shader_info_log(vertex_shader);
            gl.delete_shader(vertex_shader);
            gl.delete_program(program);
            return Err(format!("Vertex shader error: {}", log));
        }

        // Compile fragment shader
        let fragment_shader = gl.create_shader(glow::FRAGMENT_SHADER).map_err(|e| {
            gl.delete_shader(vertex_shader);
            gl.delete_program(program);
            format!("Failed to create fragment shader: {}", e)
        })?;
        gl.shader_source(fragment_shader, LCD_FRAGMENT_SHADER);
        gl.compile_shader(fragment_shader);

        if !gl.get_shader_compile_status(fragment_shader) {
            let log = gl.get_shader_info_log(fragment_shader);
            gl.delete_shader(vertex_shader);
            gl.delete_shader(fragment_shader);
            gl.delete_program(program);
            return Err(format!("Fragment shader error: {}", log));
        }

        // Link shaders to program
        gl.attach_shader(program, vertex_shader);
        gl.attach_shader(program, fragment_shader);
        gl.link_program(program);

        if !gl.get_program_link_status(program) {
            let log = gl.get_program_info_log(program);
            gl.delete_shader(vertex_shader);
            gl.delete_shader(fragment_shader);
            gl.delete_program(program);
            return Err(format!("Program link error: {}", log));
        }

        // Clean up shaders after linking
        gl.delete_shader(vertex_shader);
        gl.delete_shader(fragment_shader);

        // Create vertex array and buffers
        let vao = gl.create_vertex_array().map_err(|e| {
            gl.delete_program(program);
            format!("Failed to create VAO: {}", e)
        })?;
        gl.bind_vertex_array(Some(vao));

        // Quad vertices with UV coordinates
        #[rustfmt::skip]
        let vertices: [f32; 16] = [
            // pos      // uv
            -1.0, -1.0,  0.0, 1.0,
             1.0, -1.0,  1.0, 1.0,
             1.0,  1.0,  1.0, 0.0,
            -1.0,  1.0,  0.0, 0.0,
        ];

        let indices: [u32; 6] = [0, 1, 2, 2, 3, 0];

        let vbo = gl.create_buffer().map_err(|e| {
            gl.delete_vertex_array(vao);
            gl.delete_program(program);
            format!("Failed to create VBO: {}", e)
        })?;
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
        gl.buffer_data_u8_slice(
            glow::ARRAY_BUFFER,
            bytemuck::cast_slice(&vertices),
            glow::STATIC_DRAW,
        );

        let ebo = gl.create_buffer().map_err(|e| {
            gl.delete_buffer(vbo);
            gl.delete_vertex_array(vao);
            gl.delete_program(program);
            format!("Failed to create EBO: {}", e)
        })?;
        gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
        gl.buffer_data_u8_slice(
            glow::ELEMENT_ARRAY_BUFFER,
            bytemuck::cast_slice(&indices),
            glow::STATIC_DRAW,
        );

        // Setup vertex attributes
        gl.enable_vertex_attrib_array(0);
        gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 16, 0);

        gl.enable_vertex_attrib_array(1);
        gl.vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, 16, 8);

        // Create texture
        let texture = gl.create_texture().map_err(|e| {
            gl.delete_buffer(ebo);
            gl.delete_buffer(vbo);
            gl.delete_vertex_array(vao);
            gl.delete_program(program);
            format!("Failed to create texture: {}", e)
        })?;
        gl.bind_texture(glow::TEXTURE_2D, Some(texture));
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MIN_FILTER,
            glow::NEAREST as i32,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MAG_FILTER,
            glow::NEAREST as i32,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_WRAP_S,
            glow::CLAMP_TO_EDGE as i32,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_WRAP_T,
            glow::CLAMP_TO_EDGE as i32,
        );

        // Unbind resources
        gl.bind_texture(glow::TEXTURE_2D, None);
        gl.bind_vertex_array(None);
        gl.bind_buffer(glow::ARRAY_BUFFER, None);

        Ok(GlContext {
            program,
            vao,
            vbo,
            ebo,
            texture,
        })
    }
}
