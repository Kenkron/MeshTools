use std::sync::{Arc, Mutex};
use egui::Widget;
extern crate nalgebra_glm as glm;
use bytemuck;
use eframe::egui_glow;
use egui_glow::glow;
use glm::{Vec3, Mat4};

pub type Triangle = [Vec3; 3];

const VERTEX_SHADER_SOURCE: &str = r#"
#version 330 core
layout (location = 0) in vec3 a_pos;
layout (location = 1) in vec3 a_normal;
uniform mat4 u_transformation;
uniform vec3 light_direction;
uniform vec3 ambient;
uniform vec3 diffuse;
uniform vec3 specular;
out vec3 v_color;
void main() {
    // Position
    gl_Position = u_transformation * vec4(a_pos.x, a_pos.y, a_pos.z , 1.0);
    gl_Position.z *= 0.001;

    // Color
    mat3 rotation = mat3(u_transformation);
    vec3 normal_3 = normalize(rotation * a_normal);
    float d = dot(normal_3, light_direction);
    vec3 reflection = light_direction - normal_3 * d * 2.;
    float s = max(0., dot(vec3(0.,0.,1.), normalize(reflection)));
    v_color = ambient + diffuse * max(0, -d) + specular * pow(s, 8);
}
"#;

const FRAGMENT_SHADER_SOURCE: &str = r#"
#version 330 core
precision mediump float;
in vec3 v_color;
out vec4 out_color;
void main() {
    out_color = vec4(v_color, 1.0);
}
"#;

fn create_shader_program(gl: &Arc<glow::Context>) -> Result<glow::Program, String>{
    use glow::HasContext as _;

    unsafe {
        let shader_program = gl.create_program()?;

        let shader_sources = [
            (glow::VERTEX_SHADER, VERTEX_SHADER_SOURCE),
            (glow::FRAGMENT_SHADER, FRAGMENT_SHADER_SOURCE),
        ];

        let mut shaders: Vec<glow::NativeShader> = Vec::new();
        for (shader_type, shader_source) in &shader_sources {
            let shader = gl.create_shader(*shader_type)?;
            gl.shader_source(shader, shader_source);
            gl.compile_shader(shader);
            if !gl.get_shader_compile_status(shader) {
                return Err(format!(
                    "Failed to compile shader: {}",
                    gl.get_shader_info_log(shader)));
            }
            gl.attach_shader(shader_program, shader);
            shaders.push(shader);
        }

        gl.link_program(shader_program);
        if !gl.get_program_link_status(shader_program) {
            return Err(format!("{}", gl.get_program_info_log(shader_program)));
        }

        for shader in shaders {
            gl.detach_shader(shader_program, shader);
            gl.delete_shader(shader);
        }
        return Ok(shader_program);
    }
}

/// A simple Widget to view Triangles in 3D space
///
/// Primary mouse drag rotates the model
/// Secondary mouse drag translates the model
/// Middle mouse drag scales the model
///
/// All persistent state for the transformations are stored in the
/// RenderableMesh.
pub struct MeshView {
    pub view_size: egui::Vec2,
    pub mesh: Arc<Mutex<RenderableMesh>>
}

impl MeshView {
    pub fn new(size: egui::Vec2, mesh: Arc<Mutex<RenderableMesh>>) -> Self {
        return Self {
            view_size: size,
            mesh
        };
    }
}

impl Widget for MeshView {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let (rect, response) =
            ui.allocate_exact_size(self.view_size, egui::Sense::drag());

        // Avoids division by zero for translation (and saves a bit of processing)
        if self.view_size.x * self.view_size.y == 0. {
            return response;
        }

        {
            let mut mesh = self.mesh.lock().unwrap();

            if response.dragged_by(egui::PointerButton::Primary) {
                mesh.rotate_y(-response.drag_delta().x * 0.01);
                mesh.rotate_x(-response.drag_delta().y * 0.01);
            }
            if response.dragged_by(egui::PointerButton::Secondary) {
                let matrix = mesh.combine_transformations();
                if let Some(inverse_matrix) = matrix.try_inverse() {
                    let delta4 = inverse_matrix * glm::Vec4::new(
                        2. * response.drag_delta().x / self.view_size.x,
                        -2. * response.drag_delta().y / self.view_size.y,
                        0., 0.);
                    mesh.translation += Vec3::new(delta4.x, delta4.y, delta4.z);
                }
            }
            if response.dragged_by(egui::PointerButton::Middle) {
                mesh.scale *= std::f32::consts::E.powf(-response.drag_delta().y * 0.01);
            }
        }

        let cb = egui_glow::CallbackFn::new(move |_info, _painter| {
            self.mesh.lock().unwrap().draw();
        });

        if ui.is_rect_visible(rect) {
            ui.painter().add(egui::PaintCallback {
                rect,
                callback: Arc::new(cb),
            });
        }
        return response;
    }
}

/// All of the data required to display a triangle mesh.
///
/// Automatically creates and destroys buffers and shaders.
/// Provides scaling, translation, and rotation fields,
/// as well as helper functions for rotation.
pub struct RenderableMesh {
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
    vertex_buffer: glow::Buffer,
    vertex_array: glow::VertexArray,
    triangle_count: usize,
    shader_program: glow::Program,
    gl: Arc<glow::Context>
}

/// A triangle mesh that can be rendered.
///
/// This structure contains all the data required to render a triangle mesh
/// to a glow::Context. It uses a simple phong shader with directional lighting,
/// and provides some basic fields for transformations.
impl RenderableMesh {

    /// Creates a RenderableMesh from a list of Triangles
    ///
    /// This function creates buffers and shaders for the gl context,
    /// which are cleaned up when the RenderableMesh is dropped.
    pub fn new(gl: Arc<glow::Context>, triangles: &Vec::<Triangle>) -> Result<Self, String> {
        use glow::HasContext as _;
        let mut triangle_vertices = Vec::<f32>::new();
        for t in triangles {
            // Only add triangles with non-zero area
            let cross_product = glm::cross(&(t[1] - t[0]), &(t[2] - t[0]));
            if glm::dot(&cross_product, &cross_product) > 0.0 {
                let normal = cross_product.normalize();
                for v in t {
                    triangle_vertices.append(&mut vec![v.x, v.y, v.z]);
                    triangle_vertices.append(&mut vec![normal.x, normal.y, normal.z]);
                }
            }
        }
        unsafe {
            let u8_buffer: &[u8] = bytemuck::cast_slice(&triangle_vertices[..]);
            let vertex_buffer = gl.create_buffer()?;
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vertex_buffer));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, u8_buffer, glow::STATIC_DRAW);
            let vertex_array = match gl.create_vertex_array() {
                Ok(val) => { val },
                Err(val) => {
                    // Delete the vertex buffer before erroring
                    gl.as_ref().delete_buffer(vertex_buffer);
                    return Err(val);
                }
            };
            gl.bind_vertex_array(Some(vertex_array));
            gl.enable_vertex_attrib_array(0);
            let bpv = 12; // Bytes Per Vector3
            gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, bpv * 2, 0);
            gl.enable_vertex_attrib_array(1);
            gl.vertex_attrib_pointer_f32(1, 3, glow::FLOAT, false, bpv * 2, bpv);

            return Ok(Self {
                scale: 1.,
                translation: Vec3::new(0., 0., 0.),
                rotation: Mat4::identity(),
                right_handed: true,
                light_direction: Vec3::new(-1.0, -1.0, -1.0),
                ambient: [0.1, 0.1, 0.15],
                diffuse: [0.5, 0.5, 0.45],
                specular: [0.2, 0.2, 0.2],
                vertex_buffer,
                vertex_array,
                shader_program: create_shader_program(&gl)?,
                triangle_count: triangles.len(),
                gl
            });
        }
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
    pub fn draw(&self) {
        use glow::HasContext as _;
        let transformation_matrix = self.combine_transformations();
        let transformation = transformation_matrix.as_slice().to_owned();
        unsafe {
            self.gl.enable(glow::DEPTH_TEST);
            if self.right_handed {
                self.gl.depth_range_f32(1., -1.);
            } else {
                self.gl.depth_range_f32(-1., 1.);
            }
            self.gl.clear(glow::DEPTH_BUFFER_BIT);
            self.gl.use_program(Some(self.shader_program));
            self.gl.uniform_matrix_4_f32_slice(
                self.gl.get_uniform_location(self.shader_program, "u_transformation").as_ref(),
                false,
                &transformation,
            );
            self.gl.uniform_3_f32_slice(
                self.gl.get_uniform_location(self.shader_program, "light_direction").as_ref(),
                self.light_direction.normalize().as_slice());
            self.gl.uniform_3_f32_slice(
                self.gl.get_uniform_location(self.shader_program, "ambient").as_ref(),
                self.ambient.as_slice());
            self.gl.uniform_3_f32_slice(
                self.gl.get_uniform_location(self.shader_program, "diffuse").as_ref(),
                self.diffuse.as_slice());
            self.gl.uniform_3_f32_slice(
                self.gl.get_uniform_location(self.shader_program, "specular").as_ref(),
                self.specular.as_slice());
            self.gl.bind_vertex_array(Some(self.vertex_array));
            self.gl.draw_arrays(glow::TRIANGLES, 0, self.get_triangle_count() as i32 * 3);
        }
    }
    /// Reference to the glow::Context used to create this mesh's buffers and shaders
    pub fn get_gl(&self) -> Arc<glow::Context> {
        return self.gl.to_owned();}
    /// The number of triangles in the vertex buffer
    pub fn get_triangle_count(&self) -> usize{
        return self.triangle_count;}
    /// Sets the rotation matrix back to the identity matrix
    pub fn reset_rotation(&mut self) {
        self.rotation = Mat4::identity();}
    /// Rotate around the x axis (relative to the model's current rotation)
    pub fn rotate_x(&mut self, radians: f32) {
        self.rotation = glm::rotate_x(&self.rotation, radians);}
    /// Rotate around the y axis (relative to the model's current rotation)
    pub fn rotate_y(&mut self, radians: f32) {
        self.rotation = glm::rotate_y(&self.rotation, radians);}
    /// Rotate around the z axis (relative to the model's current rotation)
    pub fn rotate_z(&mut self, radians: f32) {
        self.rotation = glm::rotate_z(&self.rotation, radians);}
}

impl Drop for RenderableMesh {
    fn drop(&mut self) {
        use glow::HasContext as _;
        unsafe {
            self.gl.as_ref().delete_vertex_array(self.vertex_array);
            self.gl.as_ref().delete_buffer(self.vertex_buffer);
            self.gl.as_ref().delete_program(self.shader_program);
        }
    }
}
