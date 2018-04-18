//! The actual backend (i.e., the magic happens here).

use gui;
use std::sync::{mpsc, Arc};
use std::thread;

pub mod data;

/// A 2D, integer point
pub type Point = (i32, i32);

pub fn point_minmax(a: Point, b: Point) -> (Point, Point) {
    if a.0 < b.0 {
        if a.1 < b.1 {
            ((a.0, a.1), (b.0, b.1))
        } else {
            ((a.0, b.1), (b.0, a.1))
        }
    } else {
        if a.1 < b.1 {
            ((b.0, a.1), (a.0, b.1))
        } else {
            ((b.0, b.1), (a.0, a.1))
        }
    }
}

use self::data::{AABB, QTree};

pub struct Controller {
    gui: gui::GUI,
    data: Arc<QTree>,
    send: mpsc::Sender<Arc<QTree>>,
    rendering: thread::JoinHandle<()>,
}

impl Controller {
    pub fn new() -> Controller {
        let gui = gui::GUI::new();
        let (send, recv) = mpsc::channel();

        let rendering = thread::spawn(|| {
            gui.render_loop(recv);
        });
        Controller {
            gui,
            data: Arc::new(QTree::new(AABB::new((0, 0), 4), &vec![(0, 0), (1, 0), (2, 0), (0, 1), (1, 2)])),
            send,
            rendering,
        }
    }

    pub fn run(self) {
        while !self.win.window.should_close() {

        }
        self.rendering.join().expect("Couldn't join rendering thread!");
    }
}

#[cfg(test)]
#[allow(unused_results)]
mod tests {
    use super::*;

    mod functions {
        use super::{Point, point_minmax};

        fn minmax_check(points: (Point, Point)) {
            let (a, b) = points;
            assert_eq!(a, (-1, -1));
            assert_eq!(b, (2, 3));
        }

        #[test]
        fn minmax() {
            minmax_check(point_minmax((-1, -1), (2, 3)));
            minmax_check(point_minmax((-1, 3), (2, -1)));
            minmax_check(point_minmax((2, -1), (-1, 3)));
            minmax_check(point_minmax((2, 3), (-1, -1)));
        }
    }
}
