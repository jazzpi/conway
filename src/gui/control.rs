extern crate gl;

use gui::shader::Shader;

pub struct Control {
    shader: Shader,
    vao: u32,
    vbo: u32,
}

impl Control {
    pub fn new() -> Control {
        let mut shader = Shader::new(
            "resource/shaders/control.vert", None, None, None,
            Some("resource/shaders/control.frag"), None
        ).unwrap();
        shader.use_program();
        let vao = shader.create_vao();
        let vbo = shader.create_vbo();
        shader.bind_vao(vao);
        shader.bind_vbo(gl::ARRAY_BUFFER, vbo);
        Self::buffer_data(&shader);
        shader.bind_vbo(gl::ARRAY_BUFFER, 0);
        shader.bind_vao(0);

        Control {
            shader,
            vao,
            vbo,
        }
    }

    fn buffer_data(shader: &Shader) {
        // let vertices: [f32; 8]
    }
}
