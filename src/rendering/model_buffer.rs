use std::sync::Arc;

use eframe::glow;
use glow::HasContext as _;

extern crate nalgebra_glm as glm;

enum ModelComponent<'a> {
    Float(&'a Vec<f32>),
    Vec2(&'a Vec<glm::Vec2>),
    Vec3(&'a Vec<glm::Vec3>)
}

pub struct ModelBuffer {
    pub vertex_buffer: glow::Buffer,
    pub vertex_array: glow::VertexArray,
    pub gl: Arc<glow::Context>
}

struct AttributeBuilder<'a> {
    attributes: Vec<ModelComponent<'a>>,
    gl: Arc<glow::Context>
}

impl AttributeBuilder<'_> {
    fn add_attribute(&self, component: ModelComponent) -> &AttributeBuilder {
        return self;
    }
    pub fn finish(&self) -> Result<Arc<ModelBuffer>, String> {
        let gl = self.gl;
        let mut vertex_count = 0 as usize;
        let mut bytes_per_vertex = 0;
        for component in self.attributes {
            match component {
                ModelComponent::Float(f) => {
                    vertex_count = f.len();
                    bytes_per_vertex += 4;
                },
                ModelComponent::Vec2(v) => {
                    vertex_count = v.len();
                    bytes_per_vertex += 8;
                },
                ModelComponent::Vec3(v) => {
                    vertex_count = v.len();
                    bytes_per_vertex += 12;

                }
            }
        }
        let mut float_list = Vec::<f32>::new();
        for component in self.attributes {
            match component {
                ModelComponent::Float(f) => {
                    float_list.append(&mut f);
                },
                ModelComponent::Vec2(v) => {
                    for vec in v {
                        float_list.push(vec[0]);
                        float_list.push(vec[1]);
                    }
                },
                ModelComponent::Vec3(v) => {
                    for vec in v {
                        float_list.push(vec[0]);
                        float_list.push(vec[1]);
                        float_list.push(vec[2]);
                    }
                }
            }
        }
        unsafe {
            let u8_buffer: &[u8] = bytemuck::cast_slice(&float_list[..]);
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
            let mut component_index = 0 as u32;
            let mut offset = 0 as i32;
            for component in self.attributes {
                match component {
                    ModelComponent::Float(f) => {
                        gl.enable_vertex_attrib_array(component_index);
                        gl.vertex_attrib_pointer_f32(
                            component_index, 1, glow::FLOAT, false, bytes_per_vertex, offset);
                        offset += 4;
                    },
                    ModelComponent::Vec2(v) => {
                        gl.enable_vertex_attrib_array(component_index);
                        gl.vertex_attrib_pointer_f32(
                            component_index, 2, glow::FLOAT, false, bytes_per_vertex, offset);
                        offset += 8;
                    },
                    ModelComponent::Vec3(v) => {
                        gl.enable_vertex_attrib_array(component_index);
                        gl.vertex_attrib_pointer_f32(
                            component_index, 3, glow::FLOAT, false, bytes_per_vertex, offset);
                        offset += 12;
                    }
                }
                component_index += 1;
            }
            return Ok(Arc::new(ModelBuffer{
                vertex_buffer,
                vertex_array,
                gl
            }));
        }
    }
}

pub fn build_model_buffer() {

}