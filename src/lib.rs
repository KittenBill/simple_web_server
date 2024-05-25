use std::{
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
};

pub struct ThreadPool {
    threads: Vec<Worker>,
    sender: Option<Sender<Job>>,
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
            threads: threads,
            sender: Some(sender),
        }
    }

    pub fn execute<F>(&self, f: F) -> ()
    where
        F: FnOnce() -> () + Send + 'static,
    {
        let f = Box::new(f);
        if let Some(sender) = &self.sender {
            sender.send(f).unwrap();
        } else {
            panic!("sender has been dropped.");
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take().unwrap());

        for worker in &mut self.threads {
            if let Some(thread) = worker.thread.take() {
                println!("ThreadPool is waiting for Worker {} to finish", worker.id);
                thread.join().unwrap();
                println!("Woker {} has finished.", worker.id);
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<Receiver<Job>>>) -> Self {
        let thread = thread::spawn(move || loop {
            match receiver.lock().unwrap().recv() {
                Ok(job) => {
                    println!("Worker {} received a job", id);
                    job();
                }
                Err(e) => {
                    println!("Error: {:?}. Sender is dropped, worker {} stops.", e, id);
                    break;
                }
            }
        });
        Self {
            id,
            thread: Some(thread),
        }
    }
}

type Job = Box<dyn FnOnce() -> () + Send + 'static>;
