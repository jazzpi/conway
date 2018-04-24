extern crate conway;
use conway::*;

fn main() {
    let controller = backend::Controller::new();
    controller.run();
}
