use std::{
    mem,
    ops::{Add, Mul},
    path::Path,
    ptr,
};

use gl::types::{GLboolean, GLfloat, GLsizei, GLsizeiptr};
use glam::{Mat4, Vec2, Vec3, Vec4};
use sdl2::rect::Rect;

use crate::shader::{self, Shader, ShaderLoader};

pub struct UIPayload(pub f32, pub Vec2);

pub struct UIShader {
    vertices: [GLfloat; 18],
    pub shader: Shader,
    location_size: i32,
    location_color: i32,
    location_outline_color: i32,
    location_outline_width: i32,
    location_roundness: i32,
    location_transform: i32,
    location_position: i32,
    pub location_projection: i32,
    vao: u32,
    vbo: u32,
    pub transform: Mat4,
}

impl ShaderLoader<UIRect, UIPayload> for UIShader {
    fn new(path: &Path) -> UIShader {
        let shader = Shader::load(path);
        let mut vao = 0;
        let mut vbo = 0;
        let vertices: [GLfloat; 18] = [
            100.0, 100.0, 0.0, // top right
            100.0, 0.0, 0.0, // bottom right
            0.0, 0.0, 0.0, // bottom left
            0.0, 0.0, 0.0, // bottom left
            0.0, 100.0, 0.0, // top left
            100.0, 100.0, 0.0, // top right
        ];
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * shader::FLOAT_SIZE) as GLsizeiptr,
                mem::transmute(&vertices[0]),
                gl::DYNAMIC_DRAW,
            );
            gl::VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE as GLboolean,
                (3 * shader::FLOAT_SIZE) as GLsizei,
                ptr::null(),
            );
            gl::EnableVertexAttribArray(0);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
        }
        shader.bind();
        println!("Loaded {}", shader.location("color"));
        UIShader {
            vertices,
            location_size: shader.location("size"),
            location_color: shader.location("color"),
            location_outline_color: shader.location("outline_color"),
            location_outline_width: shader.location("outline"),
            location_roundness: shader.location("roundness"),
            location_transform: shader.location("view"),
            location_projection: shader.location("projection"),
            location_position: shader.location("position"),
            shader: shader,
            vao,
            vbo,
            transform: Mat4::IDENTITY,
        }
    }

    fn upload(&mut self, rect: &mut UIRect, payload: UIPayload) {
        let mouse = &payload.1;
        let delta = &payload.0;
        //update
        rect.hovered = rect.is_inside(mouse);

        let hover_scale = if rect.hovered { 1.0 } else { -1.0 };
        rect.hover_time =
            (rect.hover_time + rect.hover_transition * delta * hover_scale).clamp(0.0, 1.0);

        let start_color = if rect.selected {
            rect.color + rect.selection_color
        } else {
            rect.color
        };
        let start_outline_color = if rect.selected {
            rect.outline_color + rect.selection_outline_color
        } else {
            rect.outline_color
        };
        //let start_outline_width = if rect.selected {
        //    rect.selection_outline_width
        //} else {
        //    rect.outline_width
        //};

        let fade = rect.hover_time;
        let color = start_color + TRANSPARENT.mix_alpha(&rect.hover_color, fade);
        let outline_color =
            start_outline_color + TRANSPARENT.mix_alpha(&rect.hover_outline_color, fade);
        //let outline_width = UIShader::lerp(start_outline_width, rect.hover_outline_width, fade);
        let outline_width = rect.outline_width;

        let bottom = rect.pos.y;
        let top = rect.pos.y + rect.size.y;
        let left = rect.pos.x;
        let right = rect.pos.x + rect.size.x;

        self.vertices[0] = right;
        self.vertices[1] = top;

        self.vertices[3] = right;
        self.vertices[4] = bottom;

        self.vertices[6] = left;
        self.vertices[7] = bottom;

        self.vertices[9] = left;
        self.vertices[10] = bottom;

        self.vertices[12] = left;
        self.vertices[13] = top;

        self.vertices[15] = right;
        self.vertices[16] = top;

        self.shader
            .load_location_mat4(self.location_transform, &self.transform);
        self.shader
            .load_location_vec4(self.location_color, &color.into());
        self.shader
            .load_location_vec4(self.location_outline_color, &outline_color.into());
        self.shader.load_location_float(
            self.location_roundness,
            rect.roundness.min(rect.size.x / 2.0).min(rect.size.y / 2.0),
        );
        self.shader
            .load_location_float(self.location_outline_width, outline_width);
        self.shader
            .load_location_vec2(self.location_size, &rect.size);
        self.shader
            .load_location_vec2(self.location_position, &rect.pos);

        if rect.visible {
            unsafe {
                gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
                gl::BufferSubData(
                    gl::ARRAY_BUFFER,
                    0,
                    (18 * shader::FLOAT_SIZE) as GLsizeiptr,
                    mem::transmute(&self.vertices[0]),
                );
                gl::BindVertexArray(self.vao);
                gl::DrawArrays(gl::TRIANGLES, 0, 6);
            }
        }
        let matrix = self.transform.clone();
        self.transform = Mat4::from_translation(Vec3::new(rect.pos.x, rect.pos.y, 0.0))
            .mul_mat4(&self.transform);
        for child in rect.children.iter_mut() {
            self.upload(child, UIPayload(*delta, *mouse));
        }
        self.transform = matrix;
    }
}

impl UIShader {
    pub fn set_view(&self, matrix: &Mat4) {
        self.shader.bind();

        self.shader
            .load_location_mat4(self.location_projection, matrix);
    }

    pub fn draw_rect(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        roundness: f32,
        color: Color,
    ) {
        self.draw_complete(x, y, width, height, color, TRANSPARENT, 0.0, roundness);
    }
    pub fn draw_outlined_rect(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        roundness: f32,
        color: Color,
        outline_width: f32,
        outline_color: Color,
    ) {
        self.draw_complete(
            x,
            y,
            width,
            height,
            color,
            outline_color,
            outline_width,
            roundness,
        );
    }
    
    fn draw_complete(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: Color,
        outline_color: Color,
        outline_width: f32,
        roundness: f32,
    ) {
        self.shader.bind();
        let bottom = y;
        let top = y + height;
        let left = x;
        let right = x + width;

        self.vertices[0] = right;
        self.vertices[1] = top;

        self.vertices[3] = right;
        self.vertices[4] = bottom;

        self.vertices[6] = left;
        self.vertices[7] = bottom;

        self.vertices[9] = left;
        self.vertices[10] = bottom;

        self.vertices[12] = left;
        self.vertices[13] = top;

        self.vertices[15] = right;
        self.vertices[16] = top;

        self.shader
            .load_location_mat4(self.location_transform, &self.transform);
        self.shader
            .load_location_vec4(self.location_color, &color.into());
        self.shader
            .load_location_vec4(self.location_outline_color, &outline_color.into());
        self.shader.load_location_float(
            self.location_roundness,
            roundness.min(width / 2.0).min(height / 2.0),
        );
        self.shader
            .load_location_float(self.location_outline_width, outline_width);
        self.shader
            .load_location_vec2(self.location_size, &Vec2::new(width, height));
        self.shader
            .load_location_vec2(self.location_position, &Vec2::new(x, y));

        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl::BufferSubData(
                gl::ARRAY_BUFFER,
                0,
                (18 * shader::FLOAT_SIZE) as GLsizeiptr,
                mem::transmute(&self.vertices[0]),
            );
            gl::BindVertexArray(self.vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
        }
    }
}

impl Drop for UIShader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteBuffers(1, &self.vbo);
        }
    }
}

pub struct UIRect {
    pub pos: Vec2,
    pub size: Vec2,
    pub color: Color,
    pub visible: bool,
    pub hovered: bool,
    pub roundness: f32,
    pub outline_width: f32,
    pub outline_color: Color,
    pub selected: bool,
    pub selection_outline_width: f32,
    pub selection_outline_color: Color,
    pub selection_color: Color,
    pub hover_outline_width: f32,
    pub hover_outline_color: Color,
    pub hover_color: Color,
    pub hover_transition: f32,
    pub text_color: Color,
    pub text_shadow: Color,
    pub text: Option<String>,
    pub text_align: Alignment,
    pub children: Vec<UIRect>,
    hover_time: f32,
}
#[allow(dead_code)]
impl UIRect {
    pub fn default() -> UIRect {
        UIRect {
            pos: Vec2::ZERO,
            size: Vec2::ZERO,
            color: GRAY,
            roundness: 0.0,
            visible: true,
            hovered: false,
            outline_width: 0.0,
            outline_color: TRANSPARENT,
            selected: false,
            selection_outline_width: 0.0,
            selection_outline_color: TRANSPARENT,
            selection_color: TRANSPARENT,
            hover_outline_width: 0.0,
            hover_outline_color: TRANSPARENT,
            hover_color: TRANSPARENT,
            hover_transition: 7.0,
            text_color: WHITE,
            text_shadow: TRANSPARENT,
            text: None,
            text_align: Alignment::Hidden,
            children: vec![],
            hover_time: 0.0,
        }
    }

    pub fn set_color(&mut self, base: Color, hover: Color, selection: Color) {
        self.color = base;
        self.hover_color = hover;
        self.selection_color = selection;
    }
    pub fn set_all_color(&mut self, base: Color) {
        self.set_color(base, base, base);
    }

    pub fn set_outline_color(&mut self, base: Color, hover: Color, selection: Color) {
        self.outline_color = base;
        self.hover_outline_color = hover;
        self.selection_outline_color = selection;
    }
    pub fn set_all_outline_color(&mut self, base: Color) {
        self.set_outline_color(base, base, base);
    }

    fn to_rect(&self) -> Rect {
        Rect::new(
            self.pos.x as i32,
            self.pos.y as i32,
            self.size.x as u32,
            self.size.y as u32,
        )
    }

    pub fn is_inside(&self, point: &Vec2) -> bool {
        point.x >= self.pos.x
            && point.y >= self.pos.y
            && point.x < self.pos.x + self.size.x
            && point.y < self.pos.y + self.size.y
    }
}
#[allow(dead_code)]
pub enum Alignment {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Middle,
    Hidden,
    Custom(f32, f32),
}
pub const WHITE: Color = Color::hex(0xFF_FF_FF_FF);
pub const BLACK: Color = Color::hex(0xFF_00_00_00);
pub const GRAY: Color = Color::hex(0xFF_86_86_86);
pub const TRANSPARENT: Color = Color::hex(0);
#[allow(dead_code)]
pub const RED: Color = Color::hex(0xFF_FF_00_00);
#[allow(dead_code)]
pub const GREEN: Color = Color::hex(0xFF_00_FF_00);
#[allow(dead_code)]
pub const BLUE: Color = Color::hex(0xFF_00_00_FF);

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}
#[allow(dead_code)]
impl Color {
    pub const fn rgb(red: u8, green: u8, blue: u8) -> Color {
        Color {
            red,
            green,
            blue,
            alpha: 255,
        }
    }
    pub const fn rgba(red: u8, green: u8, blue: u8, alpha: u8) -> Color {
        Color {
            red,
            green,
            blue,
            alpha,
        }
    }
    pub const fn hex(code: u32) -> Color {
        let color = Color {
            green: ((code >> 8) & 255) as u8,
            blue: (code & 255) as u8,
            red: ((code >> 16) & 255) as u8,
            alpha: ((code >> 24) & 255) as u8,
        };
        color
    }
    pub fn mix(&self, rhs: &Color, fade: f32) -> Color {
        let fade = fade.clamp(0.0, 1.0);
        let red = (self.red as i32 + ((rhs.red as i32 - self.red as i32) as f32 * fade) as i32)
            .clamp(0, 255) as u8;
        let green = (self.green as i32
            + ((rhs.green as i32 - self.green as i32) as f32 * fade) as i32)
            .clamp(0, 255) as u8;
        let blue = (self.blue as i32 + ((rhs.blue as i32 - self.blue as i32) as f32 * fade) as i32)
            .clamp(0, 255) as u8;
        Color {
            red,
            green,
            blue,
            alpha: self.alpha,
        }
    }
    pub fn mix_alpha(&self, rhs: &Color, fade: f32) -> Color {
        let fade = fade.clamp(0.0, 1.0);
        let red = (self.red as i32 + ((rhs.red as i32 - self.red as i32) as f32 * fade) as i32)
            .clamp(0, 255) as u8;
        let green = (self.green as i32
            + ((rhs.green as i32 - self.green as i32) as f32 * fade) as i32)
            .clamp(0, 255) as u8;
        let blue = (self.blue as i32 + ((rhs.blue as i32 - self.blue as i32) as f32 * fade) as i32)
            .clamp(0, 255) as u8;
        let alpha = (self.alpha as i32
            + ((rhs.alpha as i32 - self.alpha as i32) as f32 * fade) as i32)
            .clamp(0, 255) as u8;
        Color {
            red,
            green,
            blue,
            alpha,
        }
    }
}
impl Add for Color {
    type Output = Color;
    fn add(self, rhs: Self) -> Color {
        let mut mix = self.mix_alpha(&rhs, rhs.alpha as f32 / 255.0);
        mix.alpha = (self.alpha as i32 + rhs.alpha as i32).clamp(0, 255) as u8;
        mix
    }
}

impl Into<sdl2::pixels::Color> for Color {
    fn into(self) -> sdl2::pixels::Color {
        return sdl2::pixels::Color::RGBA(self.red, self.green, self.blue, self.alpha);
    }
}
impl Into<Vec4> for Color {
    fn into(self) -> Vec4 {
        return Vec4::new(
            self.red as f32 / 255.0,
            self.green as f32 / 255.0,
            self.blue as f32 / 255.0,
            self.alpha as f32 / 255.0,
        );
    }
}
mod tests {
    use crate::ui::Color;

    use super::UIShader;

    #[test]
    fn test_color() {
        let zero = Color::hex(0);
        let max = Color::rgb(255, 255, 255);
        let fade = zero.mix(&max, 0.5);
        assert_eq!(fade, Color::rgba(127, 127, 127, 0));

        let fade = max.mix_alpha(&zero, 0.5);
        assert_eq!(fade, Color::rgba(128, 128, 128, 128));

        let fade = max.mix_alpha(&zero, 0.0);
        assert_eq!(fade, Color::rgba(255, 255, 255, 255));

        let fade = max.mix_alpha(&zero, 1.0);
        assert_eq!(fade, Color::rgba(0, 0, 0, 0));
    }
    #[test]
    fn test_lerp() {
        // assert_eq!(Ma::lerp(0.0, 1.0, 1.0),1.0);
        // assert_eq!(UIShader::lerp(0.0, 1.0, 0.5),0.5);
        // assert_eq!(UIShader::lerp(0.0, 1.0, 0.0),0.0);
    }
}
