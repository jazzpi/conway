use std::sync::Arc;
use std::sync::mpsc::Sender;

use backend::data::QTree;

pub struct Updater {
    current: Arc<QTree>,
    data_send: Sender<Arc<QTree>>,
}

impl Updater {
    pub fn new(data: Arc<QTree>, data_send: Sender<Arc<QTree>>) -> Updater {
        Updater {
            current: data,
            data_send,
        }
    }

    pub fn run(mut self) {
        while self.data_send.send(Arc::clone(&self.current)).is_ok() {
            self.current = Self::build_next(&*self.current);
        }
    }

    fn build_next(current: &QTree) -> Arc<QTree> {
        let mut next = Arc::new(QTree::new(current.boundary(), &vec![]));
        {
            let data = Arc::get_mut(&mut next).unwrap();
            for point in current {
                data.set(point);
            }
        }
        next
    }
}
