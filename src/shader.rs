extern crate gl;

use std::ffi::{CStr, CString};
use std::fs::{self, File, OpenOptions};
use std::path::Path;
use std::{ptr, mem};

use gl::types::{self, GLchar, GLint, GLuint, GLfloat};

pub const FLOAT_SIZE: usize = mem::size_of::<GLfloat>();

pub struct Shader {
    program: GLuint,
}
#[allow(dead_code)]
impl Shader {
    pub fn bind(&self) {
        unsafe {
            gl::UseProgram(self.program);
        }
    }

    pub fn unbind(&self) {
        unsafe {
            gl::UseProgram(0);
        }
    }

    pub fn delete(&self) {
        unsafe {
            gl::DeleteProgram(self.program);
        }
    }

    pub fn load_mat4(&self, name: &str, mat: &glam::Mat4) {
        unsafe {
            let c_str = CString::new(name).unwrap();
            let location = gl::GetUniformLocation(self.program, c_str.as_ptr());
            gl::UniformMatrix4fv(location, 1, gl::FALSE, &mat.to_cols_array()[0]);
        }
    }

    pub fn load_vec4(&self, name: &str, vec: &glam::Vec4) {
        unsafe {
            let c_str = CString::new(name).unwrap();
            let location = gl::GetUniformLocation(self.program, c_str.as_ptr());

            gl::Uniform4fv(location, 1, &vec.to_array()[0]);
        }
    }
    pub fn load_vec2(&self, name: &str, vec: &glam::Vec2) {
        unsafe {
            let c_str = CString::new(name).unwrap();
            let location = gl::GetUniformLocation(self.program, c_str.as_ptr());

            gl::Uniform2fv(location, 1, &vec.to_array()[0]);
        }
    }
    pub fn load_float(&self, name: &str, float: f32) {
        unsafe {
            let c_str = CString::new(name).unwrap();
            let location = gl::GetUniformLocation(self.program, c_str.as_ptr());

            gl::Uniform1f(location, float as types::GLfloat);
        }
    }

    pub fn load_location_mat4(&self, location: i32, mat: &glam::Mat4) {
        unsafe {
            gl::UniformMatrix4fv(location, 1, gl::FALSE, &mat.to_cols_array()[0]);
        }
    }

    pub fn load_location_vec4(&self, location: i32, vec: &glam::Vec4) {
        unsafe {
            gl::Uniform4fv(location, 1, &vec.to_array()[0]);
        }
    }
    pub fn load_location_vec2(&self, location: i32, vec: &glam::Vec2) {
        unsafe {
            gl::Uniform2fv(location, 1, &vec.to_array()[0]);
        }
    }
    pub fn load_location_float(&self, location: i32, float: f32) {
        unsafe {
            gl::Uniform1f(location, float as types::GLfloat);
        }
    }

    pub fn location(&self, name: &str) -> i32 {
        unsafe {
            let c_str = CString::new(name).unwrap();
            let location = gl::GetUniformLocation(self.program, c_str.as_ptr());
            return location;
        }
    }

    pub fn load(location: &Path) -> Shader {
        if !location.exists() {
            panic!("Path {} does not exist", location.to_str().unwrap());
        }
        let name = location.file_name().unwrap().to_str().unwrap();
        let mut str = "".to_owned();
        str.push_str(name);

        let mut frag_path = str.clone();
        frag_path.push_str(".frag");
        let frag = location.join(Path::new(frag_path.as_str()));
        if !frag.exists() {
            panic!("Frag Shader does not exist");
        }

        let mut vert_path = str.clone();
        vert_path.push_str(".vert");
        let vert = location.join(Path::new(vert_path.as_str()));
        if !vert.exists() {
            panic!("Frag Shader does not exist");
        }

        let fragment_src = fs::read_to_string(frag).expect("Failed to read fragment shader");
        let vertex_src = fs::read_to_string(vert).expect("Failed to read vertex shader");

        let vert_link = Shader::compile(gl::VERTEX_SHADER, vertex_src.as_str());
        let frag_link = Shader::compile(gl::FRAGMENT_SHADER, fragment_src.as_str());

        let handle = Shader::link(frag_link, vert_link);

        return Shader { program: handle };
    }

    fn compile(type_: types::GLenum, src: &str) -> u32 {
        unsafe {
            let handle = gl::CreateShader(type_);
            let c_str = CString::new(src.as_bytes()).unwrap();
            gl::ShaderSource(handle, 1, &c_str.as_ptr(), ptr::null());
            gl::CompileShader(handle);

            let mut status = gl::FALSE as GLint;
            gl::GetShaderiv(handle, gl::COMPILE_STATUS, &mut status);
            if status != (gl::TRUE as GLint) {
                let mut len = 0;
                gl::GetShaderiv(handle, gl::INFO_LOG_LENGTH, &mut len);
                let mut buf = Vec::with_capacity(len as usize);
                buf.set_len((len as usize) - 1);
                gl::GetShaderInfoLog(
                    handle,
                    len,
                    ptr::null_mut(),
                    buf.as_mut_ptr() as *mut GLchar,
                );
                panic!(
                    "{}",
                    std::str::from_utf8(&buf)
                        .ok()
                        .expect("ShaderInfoLog not valid utf8")
                );
            }
            handle
        }
    }

    fn link(fragment_shader: u32, vertex_shader: u32) -> u32 {
        unsafe {
            let shader_program = gl::CreateProgram();
            gl::AttachShader(shader_program, vertex_shader);
            gl::AttachShader(shader_program, fragment_shader);
            gl::LinkProgram(shader_program);
            // check for linking errors
            let mut status = gl::FALSE as GLint;
            gl::GetProgramiv(shader_program, gl::LINK_STATUS, &mut status);
            if status != (gl::TRUE as GLint) {
                let mut len: GLint = 0;
                gl::GetProgramiv(shader_program, gl::INFO_LOG_LENGTH, &mut len);
                let mut buf = Vec::with_capacity(len as usize);
                buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
                gl::GetProgramInfoLog(
                    shader_program,
                    len,
                    ptr::null_mut(),
                    buf.as_mut_ptr() as *mut GLchar,
                );
                panic!(
                    "{}",
                    std::str::from_utf8(&buf)
                        .ok()
                        .expect("ProgramInfoLog not valid utf8")
                );
            }
            gl::DeleteShader(vertex_shader);
            gl::DeleteShader(fragment_shader);
            shader_program
        }
    }
}
impl Drop for Shader {
    fn drop(&mut self) {
        self.delete();
    }
}

pub trait ShaderLoader<T, D> where Self: Drop {
    fn upload(&mut self, object: &mut T, payload: D);
    fn new(shader: &Path) -> Self;
}
