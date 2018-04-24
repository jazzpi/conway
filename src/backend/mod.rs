//! The actual backend (i.e., the magic happens here).

use gui;
use std::sync::{mpsc, Arc};
use std::thread;

pub mod data;
mod updater;
use self::updater::Updater;

/// A 2D, integer point
pub type Point = (i32, i32);

/// Creates a "minimal" and "maximal" point from two points.
///
/// The minimal point will have the minimal x and minimal y coordinates of
/// a and b (and vice versa for the maximal point).
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

/// The controller glues the whole game together and controls the logic flow.
///
/// **TODO:** Should this be in backend?
pub struct Controller {
    gui: gui::GUI,
    updater: thread::JoinHandle<()>,
}

impl Controller {
    /// Constructs a new controller.
    ///
    /// **Note:** Since we construct the GUI in here, this _must_ be called
    /// from the main thread.
    pub fn new() -> Controller {
        let (data_send, data_recv) = mpsc::channel();

        let data = Arc::new(QTree::new(
            AABB::new((0, 0), 4),
            &vec![(0, 0), (1, 0), (2, 0), (0, 1), (1, 2)]
        ));

        let gui = gui::GUI::new(data_recv);

        let updater = thread::spawn(|| {
            Updater::new(data, data_send).run();
        });

        Controller {
            gui,
            updater,
        }
    }

    /// Runs the game.
    ///
    /// **Note:** Since we poll GLFW events in here, this _must_ be called
    /// from the main thread.
    pub fn run(self) {
        self.gui.run();
        self.updater.join().expect("Couldn't join updater!");
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
