mod renderer;
mod shaders;

pub use renderer::{GlContext, render_with_shader};
pub use shaders::init_gl_context;
