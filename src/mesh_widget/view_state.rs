use eframe::{egui_glow::glow, glow::HasContext};

extern crate nalgebra_glm as glm;
use std::sync::Arc;

use glm::{Vec3, Mat4};

use super::{GlowState, Triangle};

/// All of the data required to display a triangle mesh.
///
/// Provides scaling, translation, and rotation fields,
/// as well as helper functions for rotation.
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
    glow_state: Arc<GlowState>
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
        let largest_dim = get_largest_dim(triangles);
        let mut scale = 1.0;
        if largest_dim != 0.0 {
            scale = 1.0/largest_dim;
        }
        return Ok(Self {
            scale,
            translation: -get_center(triangles),
            rotation: Mat4::identity(),
            right_handed: true,
            light_direction: Vec3::new(-1.0, -1.0, -1.0),
            ambient: [0.1, 0.1, 0.15],
            diffuse: [0.5, 0.5, 0.45],
            specular: [0.2, 0.2, 0.2],
            glow_state: GlowState::new(gl, triangles)?
        });
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
        let transformation_matrix = self.combine_transformations();
        let transformation = transformation_matrix.as_slice().to_owned();
        let glow_state = &self.glow_state;
        let gl = &glow_state.gl;
        unsafe {
            gl.enable(glow::DEPTH_TEST);
            if self.right_handed {
                gl.depth_range_f32(1., -1.);
            } else {
                gl.depth_range_f32(-1., 1.);
            }
            gl.clear(glow::DEPTH_BUFFER_BIT);
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
    /// Draws the model to an RGBA pixel buffer
    pub fn draw_pixels(&self, width: usize, height: usize) -> Result<Vec<u8>, String> {
        let glow_state = &self.glow_state;
        let gl = &glow_state.gl;
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
        return self.glow_state.gl.to_owned();}
    /// The number of triangles in the vertex buffer
    #[allow(dead_code)]
    pub fn get_triangle_count(&self) -> usize{
        return self.glow_state.triangle_count;}
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

impl Clone for ViewState {
    fn clone(&self) -> Self {
        Self {
            translation: self.translation.clone(),
            scale: self.scale.clone(),
            rotation: self.rotation.clone(),
            right_handed: self.right_handed.clone(),
            light_direction: self.light_direction.clone(),
            ambient: self.ambient.clone(),
            diffuse: self.diffuse.clone(),
            specular: self.specular.clone(),
            glow_state: self.glow_state.clone() }
    }
}

fn get_largest_dim(mesh: &Vec<Triangle>) -> f32 {
    if mesh.len() == 0 {
        return 0.;
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
    let size = max_vec - min_vec;
    return f32::max(size[0], f32::max(size[1], size[2]));
}

fn get_center(mesh: &Vec<Triangle>) -> Vec3{
    if mesh.len() == 0 {
        return Vec3::new(0.,0.,0.);
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
    return (min_vec + max_vec) / 2.0;
}