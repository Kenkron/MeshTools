use std::sync::Arc;

use eframe::egui_glow::glow;
extern crate nalgebra_glm as glm;

use super::Triangle;

pub struct GlowState {
    pub vertex_buffer: glow::Buffer,
    pub vertex_array: glow::VertexArray,
    pub triangle_count: usize,
    pub shader_program: glow::Program,
    pub gl: Arc<glow::Context>
}

impl GlowState {
    /// Creates a GlowState from a list of Triangles
    ///
    /// This function creates buffers and shaders for the gl context,
    /// which are cleaned up when the GlowState is dropped.
    ///
    /// A successful result is wrapped in an Arc to allow a clear
    /// way to clone this state without risking the GL data being destroyed
    /// while there is still a copy of the state being used.
    pub fn new(gl: Arc<glow::Context>, triangles: &Vec::<Triangle>) -> Result<Arc<Self>, String> {
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

            return Ok(Arc::new(Self {
                vertex_buffer,
                vertex_array,
                shader_program: create_shader_program(&gl)?,
                triangle_count: triangles.len(),
                gl
            }));
        }
    }
}

impl Drop for GlowState {
    fn drop(&mut self) {
        use glow::HasContext as _;
        unsafe {
            self.gl.as_ref().delete_vertex_array(self.vertex_array);
            self.gl.as_ref().delete_buffer(self.vertex_buffer);
            self.gl.as_ref().delete_program(self.shader_program);
        }
    }
}

const VERTEX_SHADER_SOURCE: &str = r#"
#version 330 core
layout (location = 0) in vec3 a_pos;
layout (location = 1) in vec3 a_normal;
uniform mat4 u_transformation;
uniform vec3 light_direction;
uniform vec3 ambient;
uniform vec3 diffuse;
uniform vec3 specular;
uniform float aspect_ratio;
out vec3 v_color;
void main() {
    // Position
    gl_Position = u_transformation * vec4(a_pos.x, a_pos.y, a_pos.z , 1.0);
    gl_Position.x /= aspect_ratio;
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
