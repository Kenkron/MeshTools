use std::sync::Arc;

use eframe::glow;
use glow::HasContext as _;

extern crate nalgebra_glm as glm;

pub struct ShaderProgram {
    pub shader_program: glow::Program,
    pub gl: Arc<glow::Context>
}

pub enum Uniform {
    Float(f32),
    Vec2(glm::Vec2),
    Vec3(glm::Vec3),
    Vec4(glm::Vec4),
    Mat3(glm::Mat3),
    Mat4(glm::Mat4)
}

impl ShaderProgram {
    pub fn new(gl: Arc<glow::Context>, vertex_source: &str, fragment_source: &str)
    -> Result<Self, String> {
        unsafe {
            let shader_program = gl.create_program()?;

            let shader_sources = [
                (glow::VERTEX_SHADER, vertex_source),
                (glow::FRAGMENT_SHADER, fragment_source),
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
                return Err(
                    format!("{}", gl.get_program_info_log(shader_program)));
            }

            for shader in shaders {
                gl.detach_shader(shader_program, shader);
                gl.delete_shader(shader);
            }
            return Ok(Self{shader_program, gl});
        }
    }
    pub fn uniform(&self, name: &str, value: Uniform) {
        let gl = self.gl;
        unsafe {
            gl.use_program(Some(self.shader_program));
            let location = gl
                .get_uniform_location(self.shader_program, name)
                .as_ref();
            match value {
                Uniform::Float(f) => {
                    gl.uniform_1_f32(location, f);
                },
                Uniform::Vec2(vec) => {
                    gl.uniform_2_f32_slice(location, vec.as_slice());
                },
                Uniform::Vec3(vec) => {
                    gl.uniform_3_f32_slice(location, vec.as_slice());
                },
                Uniform::Vec4(vec) => {
                    gl.uniform_4_f32_slice(location, vec.as_slice());
                },
                Uniform::Mat3(mat) => {
                    gl.uniform_matrix_3_f32_slice(location, false, mat.as_slice());
                },
                Uniform::Mat4(mat) => {
                    gl.uniform_matrix_4_f32_slice(location, false, mat.as_slice());
                }
            }
        }
    }
    pub fn bind(&self) {
        unsafe {
            self.gl.use_program(Some(self.shader_program));
        }
    }
}

impl Drop for ShaderProgram {
    fn drop(&mut self) {
        unsafe {
            self.gl.as_ref().delete_program(self.shader_program);
        }
    }
}

const PHONG_VERTEX_SHADER: &str = r#"
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

const SIMPLE_FRAGMENT_SHADER: &str = r#"
#version 330 core
precision mediump float;
in vec3 v_color;
out vec4 out_color;
void main() {
    out_color = vec4(v_color, 1.0);
}
"#;