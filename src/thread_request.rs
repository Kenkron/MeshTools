use std::sync::{Arc, RwLock};
use std::thread;
use std::thread::JoinHandle;

pub struct Request<T: Send + Sync + 'static> {
    res: Arc<RwLock<Option<T>>>,
    handle: JoinHandle<()>
}

impl<T: Send + Sync + 'static> Request<T> {
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce() -> T + Send + 'static,
    {
        let res = Arc::new(RwLock::new(Option::<T>::None));
        return Self {
            res: res.clone(),
            handle: thread::spawn(move || {
                let thread_result = f();
                *res.write().unwrap() = Some(thread_result);
            })
        }
    }

    pub fn result(&self) -> Arc<RwLock<Option<T>>> {
        return self.res.clone();
    }
}