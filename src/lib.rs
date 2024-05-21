use std::{
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread::{self, JoinHandle, Thread},
};

pub struct ThreadPool {
    pool_size: usize,
    threads: Vec<Worker>,
    sender: Sender<Job>,
}

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    pub fn new(size: usize) -> Self {
        assert!(size > 0);

        let (sender, reciever) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(reciever));

        let mut threads = Vec::with_capacity(size);
        for idx in 0..size {
            threads.push(Worker::new(idx, receiver.clone()));
        }
        ThreadPool {
            pool_size: size,
            threads: threads,
            sender,
        }
    }

    pub fn execute<F>(&self, f: F) -> ()
    where
        F: FnOnce() -> () + Send + 'static,
    {
        let f = Box::new(f);
        self.sender.send(f).unwrap();
    }
}

impl Drop for ThreadPool{
    fn drop(&mut self) {
        
    }
}

struct Worker {
    id: usize,
    thread: JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<Receiver<Job>>>) -> Self {
        let thread = thread::spawn(move || loop {
            while let Ok(job) = receiver.lock().unwrap().try_recv() {
                println!("Worker {} got a job, executing.", id);
                job();
            }
        });
        Self { id, thread }
    }
}

type Job = Box<dyn FnOnce() -> () + Send + 'static>;
