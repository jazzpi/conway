use std::sync::Arc;
use std::sync::mpsc::Sender;
use std::thread::sleep;
use std::time::Duration;

use backend::data::QTree;

pub struct Updater {
    data: Arc<QTree>,
    data_send: Sender<Arc<QTree>>,
}

impl Updater {
    pub fn new(data: Arc<QTree>, data_send: Sender<Arc<QTree>>) -> Updater {
        Updater {
            data,
            data_send,
        }
    }

    pub fn run(self) {
        while self.data_send.send(Arc::clone(&self.data)).is_ok() {
            sleep(Duration::from_millis(300));
        }
    }
}
