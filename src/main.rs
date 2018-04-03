extern crate conway;
use conway::*;

extern crate glfw;
use glfw::Context;

fn main() {
    let (mut win, mut renderer) = gui::init();

    while !win.window.should_close() {
        renderer.draw();
        win.window.swap_buffers();
        for ev in win.get_events() {
            println!("{:?}", ev);
            match ev {
                gui::Event::FramebufferSize(width, height) => {
                    renderer.set_viewport(width, height);
                }
                _ => {}
            }
        }
    }
}
