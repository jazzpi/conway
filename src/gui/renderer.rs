extern crate gl;

use std::ptr;
use std::mem::size_of;
use std::os::raw::c_void;
use std::ffi::CString;

use gui::shader::Shader;
use backend::Point;

const DEFAULT_WIDTH: f32 = 600.0;
const DEFAULT_HEIGHT: f32 = 600.0;
const DEFAULT_WIDTH_IN_CELLS: f32 = 20.0;
const DEFAULT_HEIGHT_IN_CELLS: f32 = 20.0;

/// Renderer that handles the actual rendering
pub struct Renderer {
    game_shader: Shader,
    game_vao: u32, game_vbo: u32,
    grid_shader: Shader,
    grid_vao: u32, grid_vbo: u32,
    grid_vertices: Vec<f32>,
    viewport: Viewport,
}

impl Renderer {
    /// Create a new Renderer and initialize the shaders.
    pub fn new() -> Renderer {
        let mut game_shader = Shader::new(
            "resource/shaders/game.vert", None, None, None,
            Some("resource/shaders/game.frag"), None
        ).unwrap();
        game_shader.use_program();
        let game_vao = game_shader.create_vao();
        let game_vbo = game_shader.create_vbo();

        let mut grid_shader = Shader::new(
            "resource/shaders/grid.vert", None, None, None,
            Some("resource/shaders/grid.frag"), None
        ).unwrap();
        grid_shader.use_program();
        let grid_vao = grid_shader.create_vao();
        let grid_vbo = grid_shader.create_vbo();

        let mut renderer = Renderer {
            game_shader,
            game_vao, game_vbo,
            grid_shader,
            grid_vao, grid_vbo,
            grid_vertices: vec![],
            viewport: Viewport::new(),
        };

        renderer.buffer_data();
        renderer.set_zoom(1.0);

        renderer.game_shader.use_program();
        renderer.game_shader.bind_vbo(gl::ARRAY_BUFFER, 0);
        renderer.game_shader.bind_vao(0);
        renderer.grid_shader.use_program();
        renderer.grid_shader.bind_vbo(gl::ARRAY_BUFFER, 0);
        renderer.grid_shader.bind_vao(0);

        renderer
    }

    fn buffer_data(&mut self) {
        self.game_shader.use_program();
        let vertices: [f32; 8] = [
            0.0, 1.0, // left top
            0.0, 0.0, // left bottom
            1.0, 0.0, // right bottom
            1.0, 1.0, // right top
        ];
        self.game_shader.bind_vao(self.game_vao);
        self.game_shader.bind_vbo(gl::ARRAY_BUFFER, self.game_vbo);
        unsafe {
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * size_of::<f32>()) as isize,
                &vertices[0] as *const f32 as *const c_void,
                gl::STATIC_DRAW
            );
        }
        Self::setup_vao(&mut self.game_shader, "pos");
    }

    fn setup_vao(shader: &mut Shader, location: &str) {
        let location = shader.get_attrib_location(
            &CString::new(location).unwrap()
        ) as u32;
        unsafe {
            gl::VertexAttribPointer(
                location,
                2, gl::FLOAT, gl::FALSE, 2 * size_of::<f32>() as i32,
                ptr::null()
            );
            gl::EnableVertexAttribArray(location);
        }
    }

    fn update_grid(&mut self) {
        self.grid_shader.use_program();

        let viewport = &self.viewport.viewport;
        let min_x = (viewport.0).0;
        let max_x = (viewport.1).0;
        let min_y = (viewport.0).1;
        let max_y = (viewport.1).1;
        let x_dim = (max_x - min_x + 1) as usize;
        let y_dim = (max_y - min_y + 1) as usize;
        // 2 floats/vertex, 2 vertices per line, xy_dim lines per dimension
        self.grid_vertices = Vec::<f32>::with_capacity(2 * 2 * (x_dim + y_dim));
        for x in min_x..(max_x + 1) {
            self.grid_vertices.push(x as f32);
            self.grid_vertices.push(min_y as f32);
            self.grid_vertices.push(x as f32);
            self.grid_vertices.push(max_y as f32);
        }
        for y in min_y..(max_y + 1) {
            self.grid_vertices.push(min_x as f32);
            self.grid_vertices.push(y as f32);
            self.grid_vertices.push(max_x as f32);
            self.grid_vertices.push(y as f32);
        }
        self.grid_shader.bind_vao(self.grid_vao);
        self.grid_shader.bind_vbo(gl::ARRAY_BUFFER, self.grid_vbo);
        unsafe {
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (self.grid_vertices.len() * size_of::<f32>()) as isize,
                &self.grid_vertices[0] as *const f32 as *const c_void,
                gl::STATIC_DRAW
            );
        }
        Self::setup_vao(&mut self.grid_shader, "pos");
    }

    /// Set the zoom level (the higher, the further out we zoom)
    pub fn set_zoom(&mut self, zoom: f32) {
        self.game_shader.use_program();
        self.game_shader.set_f32(&CString::new("zoom").unwrap(), zoom);
        self.grid_shader.use_program();
        self.grid_shader.set_f32(&CString::new("zoom").unwrap(), zoom);
        self.viewport.set_zoom(zoom);
        self.update_grid();
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
        self.game_shader.use_program();
        self.game_shader.set_i32_v2(&CString::new("viewport").unwrap(),
                                    (width, height));
        self.grid_shader.use_program();
        self.grid_shader.set_i32_v2(&CString::new("viewport").unwrap(),
                                    (width, height));

        self.viewport.set_window(width as u32, height as u32);
        self.update_grid();
    }

    /// Actually draw to the buffer
    pub fn draw(&mut self) {
        let indices: [u32; 6] = [0, 1, 2, 0, 2, 3];
        unsafe {
            gl::ClearColor(0.2, 0.2, 0.2, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            self.game_shader.use_program();
            self.game_shader.bind_vao(self.game_vao);
            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT,
                             &indices[0] as *const u32 as *const c_void);
            self.grid_shader.use_program();
            self.grid_shader.bind_vao(self.grid_vao);
            gl::DrawArrays(gl::LINES, 0, (self.grid_vertices.len() / 2) as i32);
        }
    }
}

struct Viewport {
    window_size: (f32, f32),
    zoom: f32,
    world_center: Point,
    /// Viewport _in world coordinates_
    pub viewport: (Point, Point),
}

impl Viewport {
    pub fn new() -> Viewport {
        Viewport {
            window_size: (0.0, 0.0),
            zoom: 1.0,
            world_center: (0, 0),
            viewport: ((0, 0), (0, 0)),
        }
    }

    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom;
        self.update();
    }

    pub fn set_window(&mut self, width: u32, height: u32) {
        self.window_size = (width as f32, height as f32);
        self.update();
    }

    fn update(&mut self) {
        let dim = ((self.window_size.0 * DEFAULT_WIDTH_IN_CELLS / DEFAULT_WIDTH / self.zoom).ceil(),
                   (self.window_size.1 * DEFAULT_HEIGHT_IN_CELLS / DEFAULT_HEIGHT / self.zoom).ceil());
        let dim = (dim.0 as i32, dim.1 as i32);
        self.viewport = (
            (self.world_center.0 - dim.0, self.world_center.1 - dim.1),
            (self.world_center.0 + dim.0, self.world_center.1 + dim.1),
        );
    }
}
