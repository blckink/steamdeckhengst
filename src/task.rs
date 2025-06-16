use crossbeam_channel::{Receiver, unbounded};
use once_cell::sync::Lazy;
use std::sync::Mutex;
use threadpool::ThreadPool;

static THREAD_POOL: Lazy<ThreadPool> = Lazy::new(|| ThreadPool::new(num_cpus::get()));

pub struct Task<T> {
    receiver: Mutex<Receiver<T>>,
}

impl<T: Send + 'static> Task<T> {
    pub fn spawn<F>(func: F) -> Self
    where
        F: FnOnce() -> T + Send + 'static,
    {
        let (tx, rx) = unbounded();
        THREAD_POOL.execute(move || {
            let result = func();
            let _ = tx.send(result);
        });
        Self {
            receiver: Mutex::new(rx),
        }
    }

    pub fn is_finished(&self) -> bool {
        let rx = self.receiver.lock().unwrap();
        !rx.is_empty()
    }

    pub fn try_join(&self) -> Option<T> {
        let rx = &mut *self.receiver.lock().unwrap();
        rx.try_recv().ok()
    }

    pub fn join(self) -> T {
        let rx = self.receiver.into_inner().unwrap();
        rx.recv().expect("Task thread panicked")
    }
}
