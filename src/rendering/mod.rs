mod renderer;
mod shader;

pub use renderer::{GlContext, render_with_shader};
pub use shader::init_gl_context;
