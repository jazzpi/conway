extern crate gl;

use std::ptr;
use std::mem::size_of;
use std::os::raw::c_void;
use std::ffi::CString;

use gui::shader::Shader;

/// Renderer that handles the actual rendering
pub struct Renderer {
    shader: Shader,
    vao: u32,
    vbo: u32,
}

impl Renderer {
    /// Create a new Renderer and initialize the shaders.
    pub fn new() -> Renderer {
        let mut shader = Shader::new(
            "resource/shaders/triangle.vert", None, None, None,
            Some("resource/shaders/triangle.frag"), None
        ).unwrap();
        shader.use_program();
        let vao = shader.create_vao();
        let vbo = shader.create_vbo();
        shader.bind_vao(vao);
        shader.bind_vbo(gl::ARRAY_BUFFER, vbo);
        Self::buffer_data(&shader);

        let mut renderer = Renderer {
            shader,
            vao,
            vbo,
        };

        renderer.set_zoom(10.0);

        renderer.shader.bind_vbo(gl::ARRAY_BUFFER, 0);
        renderer.shader.bind_vao(0);

        renderer
    }

    /// TODO: This is just a placeholder
    pub fn bind_vbo(&mut self) {
        self.shader.bind_vbo(gl::ARRAY_BUFFER, self.vbo);
    }

    fn buffer_data(shader: &Shader) {
        let vertices: [f32; 8] = [
            0.0, 1.0, // left top
            0.0, 0.0, // left bottom
            1.0, 0.0, // right bottom
            1.0, 1.0, // right top
        ];
        unsafe {
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * size_of::<f32>()) as isize,
                &vertices[0] as *const f32 as *const c_void,
                gl::STATIC_DRAW
            );
            let location = shader.get_attrib_location(
                &CString::new("pos").unwrap()
            ) as u32;
            gl::VertexAttribPointer(
                location,
                2, gl::FLOAT, gl::FALSE, 2 * size_of::<f32>() as i32,
                ptr::null()
            );
            gl::EnableVertexAttribArray(location);
        }
    }

    /// Set the zoom level (the higher, the further out we zoom)
    pub fn set_zoom(&mut self, zoom: f32) {
        self.shader.set_f32(&CString::new("zoom").unwrap(), zoom);
    }

    /// Update the OpenGL viewport and FOV
    pub fn set_viewport(&mut self, width: i32, height: i32) {
        unsafe {
            if gl::Viewport::is_loaded() {
                println!("Setting viewport to {}x{}", width, height);
                gl::Viewport(0, 0, width, height);
            } else {
                println!("VIEWPORT NOT LOADED!");
            }
        }
        self.shader.set_i32_v2(&CString::new("viewport").unwrap(),
                               (width, height));
    }

    /// Actually draw to the buffer
    pub fn draw(&mut self) {
        let indices: [u32; 6] = [0, 1, 2, 0, 2, 3];
        unsafe {
            gl::ClearColor(0.2, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            self.shader.use_program();
            self.shader.bind_vao(self.vao);
            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT,
                             &indices[0] as *const u32 as *const c_void);
        }
    }
}
