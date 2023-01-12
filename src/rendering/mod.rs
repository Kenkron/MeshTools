extern crate nalgebra_glm as glm;
use glm::Vec3;

pub type Triangle = [Vec3; 3];

mod shader_program;
mod render_buffer;
mod model_buffer;
mod texture_renderer;

pub use shader_program::*;
pub use render_buffer::*;
pub use texture_renderer::*;