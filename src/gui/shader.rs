extern crate gl;

use std::ptr;
use std::fs::File;
use std::io;
use std::io::Read;
use std::convert::From;
use std::ffi::CStr;

#[derive(Debug)]
pub enum ShaderError {
    IOError(io::Error),
    CompileError(String),
}

impl From<io::Error> for ShaderError {
    fn from(err: io::Error) -> Self {
        ShaderError::IOError(err)
    }
}

/// A shader program object
pub struct Shader {
    id: u32,
}

impl Shader {
    /// Create a new shader program.
    ///
    /// The arguments are paths to the shader sources. All shaders except for
    /// the vertex shader are optional.
    pub fn new(vertex: &str, tess_control: Option<&str>,
               tess_evaluation: Option<&str>, geometry: Option<&str>,
               fragment: Option<&str>, compute: Option<&str>)
               -> Result<Shader, ShaderError> {
        let vertex = Shader::create_shader(gl::VERTEX_SHADER, vertex)?;
        let program = unsafe {
            let program = gl::CreateProgram();
            gl::AttachShader(program, vertex);
            let tess_control = Self::try_attach(program, tess_control,
                                                gl::TESS_CONTROL_SHADER)?;
            let tess_evaluation = Self::try_attach(program, tess_evaluation,
                                                   gl::TESS_EVALUATION_SHADER)?;
            let geometry = Self::try_attach(program, geometry,
                                            gl::GEOMETRY_SHADER)?;
            let fragment = Self::try_attach(program, fragment,
                                            gl::FRAGMENT_SHADER)?;
            let compute = Self::try_attach(program, compute,
                                           gl::COMPUTE_SHADER)?;
            gl::LinkProgram(program);
            Self::check_program_compilation(program)?;
            gl::DeleteShader(vertex);
            Self::try_delete(tess_control);
            Self::try_delete(tess_evaluation);
            Self::try_delete(geometry);
            Self::try_delete(fragment);
            Self::try_delete(compute);
            program
        };

        let shader = Shader {id: program};
        Ok(shader)
    }

    /// Use the program stored in this shader
    pub fn use_program(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }

    /// Create a new Vertex Array Object
    /// *Note:* The program has to be active before this is called
    pub fn create_vao(&mut self) -> u32 {
        let mut vao: u32 = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
        }
        vao
    }
    /// Bind a VAO
    /// *Note:* The program has to be active before this is called
    pub fn bind_vao(&mut self, vao: u32) {
        unsafe {
            gl::BindVertexArray(vao);
        }
    }

    /// Create a new Vertex Buffer Object
    /// *Note:* The program has to be active before this is called
    pub fn create_vbo(&mut self) -> u32 {
        let mut vbo: u32 = 0;
        unsafe {
            gl::GenBuffers(1, &mut vbo);
        }
        vbo
    }
    /// Bind a VBO
    /// *Note:* The program has to be active before this is called
    pub fn bind_vbo(&mut self, target: gl::types::GLenum, vbo: u32) {
        unsafe {
            gl::BindBuffer(target, vbo)
        }
    }

    /// Get the location of an attribute
    pub fn get_attrib_location(&self, name: &CStr) -> i32 {
        unsafe {
            gl::GetAttribLocation(self.id, name.as_ptr())
        }
    }

    /// Bind a uniform float
    /// *Note:* The program has to be active before this is called
    pub fn set_f32(&mut self, name: &CStr, value: f32) {
        unsafe {
            let location = gl::GetUniformLocation(self.id, name.as_ptr());
            gl::Uniform1f(location, value);
        }
    }
    /// Bind a uniform ivec2
    /// *Note:* The program has to be active before this is called
    pub fn set_i32_v2(&mut self, name: &CStr, value: (i32, i32)) {
        unsafe {
            let location = gl::GetUniformLocation(self.id, name.as_ptr());
            gl::Uniform2i(location, value.0, value.1);
        }
    }

    /// Get a uniform float
    pub fn get_uniform_f32(&mut self, name: &CStr) -> f32 {
        unsafe {
            let location = gl::GetUniformLocation(self.id, name.as_ptr());
            let mut tmp : [f32; 1] = [0.0];
            gl::GetUniformfv(self.id, location, tmp.as_mut_ptr());
            tmp[0]
        }
    }
    /// Get a uniform ivec2
    pub fn get_uniform_i32_v2(&mut self, name: &CStr) -> (i32, i32) {
        unsafe {
            let location = gl::GetUniformLocation(self.id, name.as_ptr());
            let mut tmp : [i32; 2] = [0, 0];
            gl::GetUniformiv(self.id, location, tmp.as_mut_ptr());
            (tmp[0], tmp[1])
        }
    }

    unsafe fn try_attach(program: u32, path: Option<&str>,
                         type_: gl::types::GLenum)
                         -> Result<Option<u32>, ShaderError> {
        if let Some(path) = path {
            let id = Shader::create_shader(type_, path)?;
            gl::AttachShader(program, id);
            Ok(Some(id))
        } else {
            Ok(None)
        }
    }

    unsafe fn try_delete(id: Option<u32>) {
        if let Some(id) = id {
            gl::DeleteShader(id);
        }
    }

    fn create_shader(type_: gl::types::GLenum, path: &str)
                     -> Result<gl::types::GLuint, ShaderError> {
        let mut code = String::new();
        let _ = File::open(path)?.read_to_string(&mut code)?;
        let raw: &[u8] = code.as_bytes();
        let ptr = raw.as_ptr() as *const i8;
        let len = raw.len() as i32;

        let id = unsafe {
            let id = gl::CreateShader(type_);
            gl::ShaderSource(id, 1, &ptr, &len);
            gl::CompileShader(id);
            Self::check_shader_compilation(id)?;
            id
        };

        Ok(id)
    }

    unsafe fn check_shader_compilation(id: u32) -> Result<(), ShaderError> {
        let mut success = gl::FALSE as gl::types::GLint;
        gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
        if success != gl::TRUE as gl::types::GLint {
            let mut len: gl::types::GLint = 0;
            gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len);
            let len = (len + 1) as usize;
            let mut log = Vec::with_capacity(len);
            log.set_len(len - 1); // Skip trailing NUL byte
            gl::GetShaderInfoLog(id, len as i32, ptr::null_mut(),
                                 log.as_mut_ptr() as *mut i8);
            let log = String::from_utf8(log).unwrap();
            Err(ShaderError::CompileError(
                format!("Shader compilation error: {}", log)
            ))
        } else {
            Ok(())
        }
    }

    unsafe fn check_program_compilation(id: u32) -> Result<(), ShaderError> {
        let mut success = gl::FALSE as gl::types::GLint;
        gl::GetProgramiv(id, gl::LINK_STATUS, &mut success);
        if success != gl::TRUE as gl::types::GLint {
            let mut len: gl::types::GLint = 0;
            gl::GetProgramiv(id, gl::INFO_LOG_LENGTH, &mut len);
            let len = (len + 1) as usize;
            let mut log = Vec::with_capacity(len);
            log.set_len(len - 1); // Skip trailing NUL byte
            gl::GetProgramInfoLog(id, len as i32, ptr::null_mut(),
                                  log.as_mut_ptr() as *mut i8);
            println!("len {}, err {:?}", len, log);
            let log = String::from_utf8(log).unwrap();
            Err(ShaderError::CompileError(
                format!("Program compilation error: {}", log)
            ))
        } else {
            Ok(())
        }
    }
}
