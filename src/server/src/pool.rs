use crate::Result;

use crossbeam_queue::SegQueue;
use std::sync::Arc;
use std::thread::{self, JoinHandle};

pub trait ThreadPool: Sized {
    fn new(threads: u32) -> Result<Self>;
    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static;
}

pub struct NaiveThreadPool {}

impl ThreadPool for NaiveThreadPool {
    fn new(_threads: u32) -> Result<Self> {
        Ok(Self {})
    }

    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        std::thread::spawn(job);
    }
}

pub struct SharedQueueThreadPool {
    queue: Arc<SegQueue<Message>>,
    handles: Vec<JoinHandle<()>>,
}

enum Message {
    Job(Box<dyn FnOnce() + Send + 'static>),
    Shutdown,
}

impl ThreadPool for SharedQueueThreadPool {
    fn new(threads: u32) -> Result<Self> {
        let queue = Arc::new(SegQueue::new());
        let mut handles = Vec::new();
        handles.reserve(threads as usize);
        for _i in 0..threads {
            let queue = queue.clone();
            handles.push(thread::spawn(move || {
                SharedQueueThreadPool::handle_jobs(queue)
            }));
        }
        Ok(Self { queue, handles })
    }

    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.queue.push(Message::Job(Box::new(job)));
    }
}

impl Drop for SharedQueueThreadPool {
    #[allow(unused_must_use)]
    fn drop(&mut self) {
        for _ in 0..self.handles.len() {
            self.queue.push(Message::Shutdown);
        }
        while let Some(handle) = self.handles.pop() {
            handle.join();
        }
    }
}

impl SharedQueueThreadPool {
    fn handle_jobs(queue: Arc<SegQueue<Message>>) {
        loop {
            match queue.pop() {
                Ok(Message::Job(f)) => f(),
                Ok(Message::Shutdown) => {
                    break;
                }
                // Queue is empty
                Err(_) => {
                    continue;
                }
            }
        }
    }
}
