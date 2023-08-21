use eframe::{egui_glow::glow, glow::HasContext};

extern crate nalgebra_glm as glm;
use std::sync::Arc;

use glm::{Vec3, Mat4, Vec4};

use super::{GlowState, Triangle, glow_state};

/// All of the data required to display a triangle mesh.
///
/// Provides scaling, translation, and rotation fields,
/// as well as helper functions for rotation.
#[derive(Clone)]
pub struct ViewState {
    /// Position of the mesh (relative to its original coordinate system)
    pub translation: Vec3,
    /// Size of the mesh during render
    pub scale: f32,
    /// Rotation matrix for the mesh.
    pub rotation: Mat4,
    pub right_handed: bool,
    pub light_direction: Vec3,
    pub ambient: [f32; 3],
    pub diffuse: [f32; 3],
    pub specular: [f32; 3],
    pub models: Vec<(Arc<GlowState>, Mat4)>,
    gl: Arc<glow::Context>
}

/// A triangle mesh that can be rendered.
///
/// This structure contains all the data required to render a triangle mesh
/// to a glow::Context. It uses a simple phong shader with directional lighting,
/// and provides some basic fields for transformations.
impl ViewState {

    /// Creates a RenderableMesh from a list of Triangles
    ///
    /// This function creates buffers and shaders for the gl context,
    /// which are cleaned up when the RenderableMesh is dropped.
    pub fn new(gl: Arc<glow::Context>, triangles: &Vec::<Triangle>) -> Result<Self, String> {
        let mut scale = 1.;
        if triangles.len() > 0 {
            let mut min_point = triangles[0][0].to_owned();
            let mut max_point = triangles[0][0].to_owned();
            for triangle in triangles {
                for vertex in triangle {
                    for i in 0..3 {
                        min_point[i] = min_point[i].min(vertex[i]);
                        max_point[i] = max_point[i].max(vertex[i]);
                    }
                }
            }
            if max_point != min_point {
                scale = 1.0/(max_point - min_point).max();
            }
        }
        return Ok(Self {
            scale,
            translation: -get_center(triangles) * scale,
            rotation: Mat4::identity(),
            right_handed: true,
            light_direction: Vec3::new(-1.0, -1.0, -1.0),
            ambient: [0.1, 0.1, 0.15],
            diffuse: [0.5, 0.5, 0.45],
            specular: [0.2, 0.2, 0.2],
            models: vec![(GlowState::new(gl.clone(), triangles)?, Mat4::identity())],
            gl
        });
    }
    
    /// Creates a renderable state with no initial models
    pub fn new_empty(gl: Arc<glow::Context>) -> Result<Self, String> {
        return Ok(Self {
            scale: 1.0,
            translation: Vec3::zeros(),
            rotation: Mat4::identity(),
            right_handed: true,
            light_direction: Vec3::new(-1.0, -1.0, -1.0),
            ambient: [0.1, 0.1, 0.15],
            diffuse: [0.5, 0.5, 0.45],
            specular: [0.2, 0.2, 0.2],
            models: Vec::<(Arc<GlowState>, Mat4)>::new(),
            gl
        });
    }
    
    /// Adds a model to this view_state
    pub fn add_model(&mut self, gl: Arc<glow::Context>, triangles: &Vec::<Triangle>) -> Result<(), String> {
        self.models.push((GlowState::new(gl, triangles)?, Mat4::identity()));
        return Ok(());
    }

    /// Combines the transformations (translation, scale, rotatioin)
    /// into a single transformation matrix.
    pub fn combine_transformations(&self) -> Mat4 {
        let scale_vec = Vec3::new(self.scale, self.scale, self.scale);
        let scale = glm::scale(&Mat4::identity(),&scale_vec);
        let translation = glm::translate(&Mat4::identity(), &self.translation);
        return
            self.rotation * scale * translation;
    }

    /// Renders the mesh to its glow::Context using its combined transformations
    /// As side effects, this enables the depth test, clears and uses the depth buffer,
    /// and sets the shader program to that of the Renderable Mesh
    pub fn draw(&self, aspect_ratio: f32) {
        if self.models.len() == 0 {
            return;
        }
        let transformation_matrix = self.combine_transformations();
        let gl = &self.gl;
        unsafe {
            gl.enable(glow::DEPTH_TEST);
            if self.right_handed {
                gl.depth_range_f32(1., -1.);
            } else {
                gl.depth_range_f32(-1., 1.);
            }
            gl.clear(glow::DEPTH_BUFFER_BIT);
            for (glow_state, local_transform) in &self.models {
                let transformation = (transformation_matrix * local_transform).as_slice().to_owned();
                gl.use_program(Some(glow_state.shader_program));
                gl.uniform_matrix_4_f32_slice(
                    gl.get_uniform_location(glow_state.shader_program, "u_transformation").as_ref(),
                    false,
                    &transformation,
                );
                gl.uniform_3_f32_slice(
                    gl.get_uniform_location(glow_state.shader_program, "light_direction").as_ref(),
                    self.light_direction.normalize().as_slice());
                gl.uniform_3_f32_slice(
                    gl.get_uniform_location(glow_state.shader_program, "ambient").as_ref(),
                    self.ambient.as_slice());
                gl.uniform_3_f32_slice(
                    gl.get_uniform_location(glow_state.shader_program, "diffuse").as_ref(),
                    self.diffuse.as_slice());
                gl.uniform_3_f32_slice(
                    gl.get_uniform_location(glow_state.shader_program, "specular").as_ref(),
                    self.specular.as_slice());
                gl.uniform_1_f32(
                    gl.get_uniform_location(glow_state.shader_program, "aspect_ratio").as_ref(),
                    aspect_ratio);
                gl.bind_vertex_array(Some(glow_state.vertex_array));
                gl.draw_arrays(glow::TRIANGLES, 0, self.get_triangle_count() as i32 * 3);
            }
        }
    }
    
    /// Draws the model to an RGBA pixel buffer
    pub fn draw_pixels(&self, width: usize, height: usize) -> Result<Vec<u8>, String> {
        let gl = &self.gl;
        unsafe {
            let framebuffer  = gl.create_framebuffer()?;
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(framebuffer));
            let gl_texture = gl.create_texture()?;
            gl.bind_texture(glow::TEXTURE_2D, Some(gl_texture));
            gl.tex_image_2d(glow::TEXTURE_2D, 0, glow::SRGB8_ALPHA8 as i32, width as i32, height as i32, 0, glow::RGBA, glow::UNSIGNED_BYTE, None);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::NEAREST as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::NEAREST as i32);
            let depth_buffer = gl.create_renderbuffer()?;
            gl.bind_renderbuffer(glow::RENDERBUFFER, Some(depth_buffer));
            gl.renderbuffer_storage(glow::RENDERBUFFER, glow::DEPTH_COMPONENT, width as i32, height as i32);
            gl.framebuffer_renderbuffer(glow::FRAMEBUFFER, glow::DEPTH_ATTACHMENT, glow::RENDERBUFFER, Some(depth_buffer));
            gl.framebuffer_texture(glow::FRAMEBUFFER, glow::COLOR_ATTACHMENT0, Some(gl_texture), 0);
            gl.draw_buffer(glow::COLOR_ATTACHMENT0);

            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(framebuffer));
            gl.viewport(0, 0, width as i32, height as i32);
            gl.clear_color(0.0, 0.0, 0.0, 0.0);
            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
            self.draw(width as f32/height as f32);
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);

            let mut buffer = vec![0 as u8; (width * height * 4) as usize];
            gl.get_tex_image(
                glow::TEXTURE_2D,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                glow::PixelPackData::Slice(buffer.as_mut_slice()));
            let mut flipped_buffer = vec![0 as u8; (width * height * 4) as usize];
            for x in 0..width as usize{
                for y in 0..height as usize{
                    let i1 = (x + width * y) * 4;
                    let i2 = (x + width * ((height - 1) - y)) * 4;
                    flipped_buffer[i1] = buffer[i2];
                    flipped_buffer[i1 + 1] = buffer[i2 + 1];
                    flipped_buffer[i1 + 2] = buffer[i2 + 2];
                    flipped_buffer[i1 + 3] = buffer[i2 + 3];
                }
            }

            gl.delete_framebuffer(framebuffer);
            gl.delete_texture(gl_texture);
            gl.delete_renderbuffer(depth_buffer);

            return Ok(flipped_buffer);
        }
    }
    /// Reference to the glow::Context used to create this mesh's buffers and shaders
    #[allow(dead_code)]
    pub fn get_gl(&self) -> Arc<glow::Context> {
        return self.gl.to_owned();}
    /// The number of triangles in the vertex buffers
    #[allow(dead_code)]
    pub fn get_triangle_count(&self) -> usize{
        let mut acc = 0;
        for (model, _) in &self.models {
            acc += model.triangle_count;
        }
        return acc;
    }
    /// Sets the rotation matrix back to the identity matrix
    #[allow(dead_code)]
    pub fn reset_rotation(&mut self) {
        self.rotation = Mat4::identity();}
    /// Rotate around the x axis (relative to the model's current rotation)
    #[allow(dead_code)]
    pub fn rotate_x(&mut self, radians: f32) {
        self.rotation = glm::rotate_x(&self.rotation, radians);}
    /// Rotate around the y axis (relative to the model's current rotation)
    #[allow(dead_code)]
    pub fn rotate_y(&mut self, radians: f32) {
        self.rotation = glm::rotate_y(&self.rotation, radians);}
    /// Rotate around the z axis (relative to the model's current rotation)
    #[allow(dead_code)]
    pub fn rotate_z(&mut self, radians: f32) {
        self.rotation = glm::rotate_z(&self.rotation, radians);}
}

fn get_bounds(mesh: &Vec<Triangle>) -> Option<(Vec3, Vec3)> {
    if mesh.len() == 0 {
        return None;
    }
    let mut min_vec = mesh[0][0];
    let mut max_vec = mesh[0][0];
    for triangle in mesh {
        for vertex in triangle {
            for i in 0..vertex.len() {
                min_vec[i] = f32::min(min_vec[i], vertex[i]);
                max_vec[i] = f32::max(min_vec[i], vertex[i]);
            }
        }
    }
    return Some((min_vec, max_vec));
}

fn get_center(mesh: &Vec<Triangle>) -> Vec3{
    if let Some((min_vec, max_vec)) = get_bounds(mesh) {
        return (min_vec + max_vec) / 2.0;
    } else {
        return Vec3::new(0.,0.,0.);
    }
}