//! All the things we need for the GUI (rendering, taking input etc.)

extern crate glfw;
use self::glfw::Context;

extern crate gl;

use std::sync::Arc;
use std::sync::mpsc::Receiver;

use backend::data::QTree;

#[derive(Clone, Debug)]
/// Indicates what modifiers are held down
pub struct Modifiers {
    mod_shift: bool,
    mod_alt: bool,
    mod_control: bool,
    mod_super: bool,
}

#[derive(Debug)]
/// Mostly a wrapper around glfw::WindowEvent, but instead of `MouseButton` and
/// `CursorPos`, we provide a `Drag` event.
pub enum Event {
    /// Thin wrapper around a Key event
    Key(glfw::Key, glfw::Scancode, glfw::Action, glfw::modifiers::Modifiers),
    /// Thin wrapper around a FramebufferSize event
    FramebufferSize(i32, i32),
    /// Thin wrapper around a Scroll event
    Scroll(f64, f64),
    /// Drag event: First tuple is the distance the mouse was dragged, second
    /// array can be indexed using a glfw::MouseButton `as usize` to see if
    /// that button was pressed while dragging, the third are the modifiers
    /// pressed while dragging
    Drag((f64, f64), [bool; 8], Modifiers),
}

/// Iterator over received `Event`s
pub struct EventIterator<'a, 'b> {
    msgs: glfw::FlushedMessages<'a, (f64, glfw::WindowEvent)>,
    cursor: &'b mut (f64, f64),
    buttons: &'b mut [bool; 8],
    mods: &'b mut Modifiers,
}

impl<'a, 'b> EventIterator<'a, 'b> {
    fn update_modifiers(mods: &mut Modifiers, key: glfw::Key, action: glfw::Action) {
        let pressed = match action {
            glfw::Action::Press | glfw::Action::Repeat => true,
            glfw::Action::Release => false,
        };
        match key {
            glfw::Key::LeftShift | glfw::Key::RightShift => {
                mods.mod_shift = pressed;
            }
            glfw::Key::LeftAlt | glfw::Key::RightAlt => {
                mods.mod_alt = pressed;
            }
            glfw::Key::LeftControl | glfw::Key::RightControl => {
                mods.mod_control = pressed;
            }
            glfw::Key::LeftSuper | glfw::Key::RightSuper => {
                mods.mod_super = pressed;
            }
            _ => {}
        }
    }
}

impl<'a, 'b> Iterator for EventIterator<'a, 'b> {
    type Item = Event;

    fn next(&mut self) -> Option<Event> {
        for ev in &mut self.msgs {
            match ev {
                (_, glfw::WindowEvent::Key(key, scancode, action, mods)) => {
                    EventIterator::update_modifiers(self.mods, key, action);
                    return Some(Event::Key(key, scancode, action, mods))
                }
                (_, glfw::WindowEvent::FramebufferSize(width, height)) => {
                    // unsafe {
                    //     if gl::Viewport::is_loaded() {
                    //         gl::Viewport(0, 0, width, height);
                    //     }
                    // }
                    return Some(Event::FramebufferSize(width, height))
                }
                (_, glfw::WindowEvent::Scroll(x, y)) => {
                    return Some(Event::Scroll(x, y))
                }
                (_, glfw::WindowEvent::MouseButton(btn, glfw::Action::Release, _)) => {
                    self.buttons[btn as usize] = false;
                }
                (_, glfw::WindowEvent::MouseButton(btn, glfw::Action::Press, _)) => {
                    self.buttons[btn as usize] = true;
                }
                (_, glfw::WindowEvent::CursorPos(x, y)) => {
                    let diff = (x - self.cursor.0, y - self.cursor.1);
                    *self.cursor = (x, y);
                    if *self.buttons != [false; 8] {
                        return Some(Event::Drag(diff, *self.buttons, (*self.mods).clone()))
                    }
                }
                (_, window_event) => {
                    eprintln!("Unknown event {:?}", window_event);
                }
            }
        }
        None
    }
}

type EventReceiver = Receiver<(f64, glfw::WindowEvent)>;

/// Represents a window, mostly handles events
pub struct Window {
    /// GLFW handle
    pub window: glfw::Window,
    events: EventReceiver,
    glfw: glfw::Glfw,
    /// Cursor position
    pub cursor: (f64, f64),
    buttons: [bool; 8],
    mods: Modifiers,
}

impl Window {
    /// Create a new window and set up GLFW event handling.
    ///
    /// Unless you want to load the OpenGL function pointers yourself, you
    /// should call init_gl() after this.
    pub fn new((width, height) : (u32, u32), title: &str) -> Window {
        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

        let (mut window, events) = glfw.create_window(width, height, title,
                                                      glfw::WindowMode::Windowed)
                                       .expect("Unable to create window!");

        // TODO: What is this?
        glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
        glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
        #[cfg(target_os = "macos")]
        glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

        window.make_current();
        window.set_key_polling(true);
        window.set_framebuffer_size_polling(true);
        window.set_scroll_polling(true);
        window.set_mouse_button_polling(true);
        window.set_cursor_pos_polling(true);

        let cursor = (0.0, 0.0);
        let buttons = [false; 8];
        let mods = Modifiers {
            mod_shift: false,
            mod_alt: false,
            mod_control: false,
            mod_super: false
        };

        Window {
            window,
            events,
            glfw,
            cursor,
            buttons,
            mods,
        }
    }

    /// Load all OpenGL function pointers
    pub fn init_gl(&mut self) -> () {
        gl::load_with(|symbol| self.window.get_proc_address(symbol) as *const _);
    }

    /// Poll window events and return an iterator over them
    /// TODO: "This is useful for event handling where the blocking behaviour of
    /// Receiver::iter is undesirable." => Can event handling be done without
    /// polling?
    pub fn get_events<'a>(&'a mut self) -> EventIterator<'a, 'a> {
        self.glfw.poll_events();
        EventIterator {
            msgs: glfw::flush_messages(&self.events),
            cursor: &mut self.cursor,
            buttons: &mut self.buttons,
            mods: &mut self.mods,
        }
    }
}

pub struct GUI {
    window: Window,
    renderer: Renderer,
    data_recv: Receiver<Arc<QTree>>,
}

impl GUI {
    pub fn new(data_recv: Receiver<Arc<QTree>>) -> GUI {
        let mut window = Window::new((600, 600), "Conway's Game of Life");
        window.init_gl();
        let renderer = Renderer::new();
        GUI {
            window,
            renderer,
            data_recv,
        }
    }

    pub fn run(mut self) {
        while !self.window.window.should_close() {
            let mut should_close = false;
            for ev in self.window.get_events() {
                println!("{:?}", ev);
                match ev {
                    Event::FramebufferSize(width, height) => {
                        self.renderer.set_viewport(width, height);
                    }
                    Event::Key(glfw::Key::Escape, _, _, mods) => {
                        if mods.is_empty() {
                            should_close = true;
                        }
                    }
                    _ => {}
                }
            }
            if should_close {
                self.window.window.set_should_close(true);
                break;
            }
            if let Ok(data) = self.data_recv.recv() {
                self.renderer.draw(&*data);
            }
            self.window.window.swap_buffers();
        }
    }
}

mod shader;
pub use self::shader::Shader;
mod renderer;
pub use self::renderer::Renderer;
