use std::{ffi::c_void, mem, ptr};

use gl::{types::*, TEXTURE0};
use glam::Mat4;
use sdl2::render::Texture;

use crate::{
    shader::{Shader, ShaderLoader, self},
    ui::{self, Color},
};

pub struct Image<'r> {
    pub texture: Texture<'r>,
}

pub struct ImageShader {
    vertices: [GLfloat; 16],
    pub shader: Shader,
    location_color: i32,
    location_texture: i32,
    location_transform: i32,
    pub location_projection: i32,
    vao: u32,
    vbo: u32,
    ebo: u32,
    transform: Mat4,
}
impl<'r> ShaderLoader<Image<'r>, Color> for ImageShader {
    fn new(shader: &std::path::Path) -> ImageShader {
        let shader = Shader::load(shader);
        let mut vao = 0;
        let mut vbo = 0;
        let mut ebo = 0;
        let vertices: [GLfloat; 16] = [
            100.0, 100.0, // top right
            1.0, 1.0, 100.0, 0.0, // bottom right
            1.0, 0.0, 0.0, 0.0, // bottom left
            0.0, 0.0, 0.0, 100.0, // top left
            0.0, 1.0,
        ];
        let indices: [u32; 6] = [0, 1, 2, 2, 3, 0];

        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
            gl::GenBuffers(1, &mut ebo);

            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * shader::FLOAT_SIZE) as GLsizeiptr,
                mem::transmute(&vertices[0]),
                gl::DYNAMIC_DRAW,
            );
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (indices.len() * shader::FLOAT_SIZE) as GLsizeiptr,
                mem::transmute(&indices[0]),
                gl::STATIC_DRAW,
            );
            gl::VertexAttribPointer(
                0,
                2,
                gl::FLOAT,
                gl::FALSE as GLboolean,
                (4 * shader::FLOAT_SIZE) as GLsizei,
                ptr::null(),
            );
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(
                1,
                2,
                gl::FLOAT,
                gl::FALSE as GLboolean,
                (4 * shader::FLOAT_SIZE) as GLsizei,
                (2 * shader::FLOAT_SIZE) as *const c_void,
            );
            gl::EnableVertexAttribArray(1);

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
        }
        shader.bind();
        ImageShader {
            vertices,
            location_color: shader.location("color"),
            location_texture: shader.location("image"),
            location_projection: shader.location("projection"),
            location_transform: shader.location("view"),
            shader,
            vao,
            vbo,
            ebo,
            transform: Mat4::IDENTITY,
        }
    }
    fn upload(&mut self, image: &mut Image<'r>, payload: Color) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            gl::Enable(gl::TEXTURE_2D);
            image.texture.gl_bind_texture();
        }
        self.shader
            .load_location_mat4(self.location_transform, &self.transform);
        self.shader
            .load_location_vec4(self.location_color, &payload.into());

        unsafe {
            // gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl::BindVertexArray(self.vao);
            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, 0 as *const c_void);
        }
        unsafe {
            gl::Disable(gl::TEXTURE_2D);
            image.texture.gl_unbind_texture()
        }
    }
}
impl Drop for ImageShader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteBuffers(1, &self.ebo);
        }
    }
}
