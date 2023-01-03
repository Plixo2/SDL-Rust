use gl::types::*;
use glam::Mat4;
use glam::Vec4;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::BlendMode;
use sdl2::render::Canvas;
use sdl2::render::RenderTarget;
use sdl2::render::Texture;
use sdl2::render::TextureCreator;
use std::ffi::c_void;
use std::fs;
use std::io;
use std::mem;
use std::path::Path;
use std::ptr;

use crate::shader;
use crate::shader::Shader;
use crate::shader::ShaderLoader;
use crate::ui;
use crate::ui::Color;

pub struct FontRenderer<'a> {
    textures: Vec<Glyph<'a>>,
    pub shader: Shader,
    location_color: i32,
    location_transform: i32,
    pub location_projection: i32,
    vao: u32,
    vbo: u32,
    ebo: u32,
    transform: Mat4,
}

struct Glyph<'a> {
    texture: Texture<'a>,
    width: f32,
    height: f32,
    offset_y: f32,
    offset_x: f32,
    space: f32,
}

impl<'a> FontRenderer<'a> {
    pub fn new<'b, T>(
        font: &Path,
        size: f32,
        shader_path: &Path,
        texture_creator: &'b TextureCreator<T>,
    ) -> FontRenderer<'b> {
        let font_bytes = fs::read(font).expect("reading font file");
        let font = fontdue::Font::from_bytes(
            font_bytes,
            fontdue::FontSettings {
                collection_index: 0,
                scale: size,
            },
        )
        .unwrap();

        let mut glyph_list = vec![];
        for index in 0..255 {
            let char = index as u8 as char;
            let (metrics, bitmap) = font.rasterize(char, size);
            let width = metrics.width;
            let height = metrics.height;
            let mut texture = texture_creator
                .create_texture_streaming(
                    PixelFormatEnum::RGBA8888,
                    width.max(1) as u32,
                    height.max(1) as u32,
                )
                .expect("creating glyph texture");
            texture.set_blend_mode(BlendMode::Add);
            texture
                .with_lock(None, |buffer, pitch| {
                    let mut iter = bitmap.iter();
                    if height == 0 || width == 0 {
                        ()
                    }
                    for y in 0..height {
                        for x in 0..width {
                            let value = iter.next().expect("next grayscale value");
                            let value = *value;
                            let offset = y * pitch + x * 4;
                            buffer[offset] = value;
                            buffer[offset + 3] = 255;
                            buffer[offset + 2] = 255;
                            buffer[offset + 1] = 255;
                        }
                    }
                    assert_eq!(iter.len(), 0);
                })
                .expect("writing to texture");
            glyph_list.push(Glyph {
                texture,
                width: metrics.width as f32,
                height: metrics.height as f32,
                offset_y: metrics.ymin as f32,
                offset_x: metrics.xmin as f32,
                space: metrics.advance_width as f32,
            });
        }

        let shader = Shader::load(shader_path);
        let mut vao = 0;
        let mut vbo = 0;
        let mut ebo = 0;
        let vertices: [GLfloat; 16] = [0.0; 16];
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

        let location_color = shader.location("color");
        let location_projection = shader.location("projection");
        let location_transform = shader.location("view");

        FontRenderer {
            textures: glyph_list,
            location_color,
            location_projection,
            location_transform,
            shader,
            vao,
            vbo,
            ebo,
            transform: Mat4::IDENTITY,
        }
    }
    pub fn set_view(&self, matrix: &Mat4) {
        self.shader.bind();

        self.shader
            .load_location_mat4(self.location_projection, matrix);
    }

    pub fn text_width(&self, text: &String) -> f32{
        let bytes = text.as_bytes();
        let mut width = 0.0;
        for char in bytes {
            width += self.textures.get(*char as usize).expect("glyph for byte").space;
        }
        width
    }

    pub fn draw<T: Into<Vec4>>(&mut self, text: &String, x: f32, y: f32, color: T) {
        let y = y - 5.0;
        self.shader.bind();
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            gl::Enable(gl::TEXTURE_2D);
        }
        self.shader
            .load_location_mat4(self.location_transform, &self.transform);
        self.shader
            .load_location_vec4(self.location_color, &color.into());

        let bytes = text.as_bytes();

        let spacing = 0.0;
        let mut x = x;
        for char in bytes {
            x += self.render(*char, x as f32, y) + spacing;
        }
        unsafe {
            gl::Disable(gl::TEXTURE_2D);
        }
    }

    fn render(&mut self, char: u8, x: f32, y: f32) -> f32 {
        let glyph = &mut self.textures.get_mut(char as usize).expect("glyph for byte");
        unsafe {
            glyph.texture.gl_bind_texture();
        }

        let width = glyph.width;
        let height = glyph.height;
        let y = y - height - glyph.offset_y;
        let x = x + glyph.offset_x;

        unsafe {
            gl::BindVertexArray(self.vao);
            let top = y + height;
            let right = x + width as f32;
            let bottom = y;
            let left = x;
            let vertices: [GLfloat; 16] = [
                right, top, // top right
                1.0, 1.0, right, bottom, // bottom right
                1.0, 0.0, left, bottom, // bottom left
                0.0, 0.0, left, top, // top left
                0.0, 1.0,
            ];
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl::BufferSubData(
                gl::ARRAY_BUFFER,
                0,
                (16 * shader::FLOAT_SIZE) as GLsizeiptr,
                mem::transmute(&vertices[0]),
            );

            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, 0 as *const c_void);
        }
        glyph.space
    }
}

impl<'a> Drop for FontRenderer<'a> {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteBuffers(1, &self.ebo);
        }
    }
}
