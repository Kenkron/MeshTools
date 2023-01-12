use std::sync::Arc;
use eframe::egui_glow::glow;
use glow::HasContext as _;

struct RenderBuffer {
    pub gl: Arc<glow::Context>,
    pub width: usize,
    pub height: usize,
    pub frame_buffer: glow::Framebuffer,
    pub texture: glow::Texture,
    pub depth_buffer: glow::Renderbuffer
}

impl RenderBuffer {
    pub fn new(gl: Arc<glow::Context>, width: usize, height: usize) -> Result<Arc<Self>, String> {
        unsafe {
            let frame_buffer  = gl.create_framebuffer()?;
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(frame_buffer));
            let texture = gl.create_texture()?;
            gl.bind_texture(glow::TEXTURE_2D, Some(texture));
            gl.tex_image_2d(glow::TEXTURE_2D, 0, glow::SRGB8_ALPHA8 as i32, width as i32, height as i32, 0, glow::RGBA, glow::UNSIGNED_BYTE, None);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::NEAREST as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::NEAREST as i32);
            let depth_buffer = gl.create_renderbuffer()?;
            gl.bind_renderbuffer(glow::RENDERBUFFER, Some(depth_buffer));
            gl.renderbuffer_storage(glow::RENDERBUFFER, glow::DEPTH_COMPONENT, width as i32, height as i32);
            gl.framebuffer_renderbuffer(glow::FRAMEBUFFER, glow::DEPTH_ATTACHMENT, glow::RENDERBUFFER, Some(depth_buffer));
            gl.framebuffer_texture(glow::FRAMEBUFFER, glow::COLOR_ATTACHMENT0, Some(texture), 0);
            gl.draw_buffer(glow::COLOR_ATTACHMENT0);

            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(frame_buffer));
            gl.viewport(0, 0, width as i32, height as i32);
            gl.clear_color(0.0, 0.0, 0.0, 0.0);
            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
            return Ok(Arc::new(Self{
                gl,
                width,
                height,
                frame_buffer,
                texture,
                depth_buffer
            }));
        }
    }
    pub fn bind(&self) {
        unsafe {
            self.gl.bind_framebuffer(glow::FRAMEBUFFER, Some(self.frame_buffer));
        }
    }
    pub fn unbind(&self) {
        unsafe {
            self.gl.bind_framebuffer(glow::FRAMEBUFFER, None);
        }
    }
    pub fn capture<R, E>(&self, render: impl FnOnce(Option<glow::Framebuffer>), old_framebuffer: Option<glow::Framebuffer>) {
        unsafe {
            self.gl.bind_framebuffer(glow::FRAMEBUFFER, Some(self.frame_buffer));
            let result = render(Some(self.frame_buffer));
            self.gl.bind_framebuffer(glow::FRAMEBUFFER, old_framebuffer);
        }
    }
    pub fn draw(&self) {

    }
    pub fn get_pixels(&self) -> Result<Vec<u8>, String> {
        let (width, height) = (self.width, self.height);
        let mut buffer = vec![0 as u8; (width * height * 4) as usize];
        unsafe {
            self.gl.get_tex_image(
                glow::TEXTURE_2D,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                glow::PixelPackData::Slice(buffer.as_mut_slice()));
        }
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
        return Ok(flipped_buffer);
    }
    pub fn get_depth_pixels(&self) -> Result<Vec<f32>, String> {
        let (width, height) = (self.width, self.height);
        let mut byte_buffer = vec![0 as u8; (width * height * 4) as usize];
        unsafe {
            self.gl.get_tex_image(
                glow::TEXTURE_2D,
                0,
                glow::DEPTH_COMPONENT,
                glow::FLOAT,
                glow::PixelPackData::Slice(byte_buffer.as_mut_slice()));
        }
        let mut buffer = vec![0.0; (width * height) as usize];
        for i in 0..buffer.len() {
            let arr = [
                byte_buffer[i*4],
                byte_buffer[i*4 + 1],
                byte_buffer[i*4 + 2],
                byte_buffer[i*4 + 3]];
            buffer[i] = f32::from_ne_bytes(arr);
        }
        let mut flipped_buffer = vec![0.0; (width * height) as usize];
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
        return Ok(flipped_buffer);
    }
}

impl Drop for RenderBuffer {
    fn drop(&mut self) {
        let gl = self.gl;
        unsafe {
            gl.delete_framebuffer(self.frame_buffer);
            gl.delete_texture(self.texture);
            gl.delete_renderbuffer(self.depth_buffer);
        }
    }
}