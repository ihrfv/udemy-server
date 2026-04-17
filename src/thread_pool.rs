use crossbeam_channel::{Receiver, Sender, unbounded};
use std::thread::{self, JoinHandle};

type Job = Box<dyn FnOnce() + Send + 'static>;

pub(crate) struct ThreadPool {
    workers: Vec<Worker>,
    // Using an Option in order to allow gracefull shutdown in Drop
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
    pub(crate) fn new(capacity: usize) -> Self {
        assert!(capacity > 0);

        let mut workers = Vec::with_capacity(capacity);

        let (tx, rx) = unbounded();

        for i in 0..capacity {
            workers.push(Worker::new(i, rx.clone()));
        }

        ThreadPool {
            workers,
            sender: Option::Some(tx),
        }
    }

    pub(crate) fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.as_ref()
            .expect("Sender should be always present all the way until the ThreadPool drop")
            .send(job)
            .expect("At least 1 receiver should exist as long as ThreadPool exists, since Workers are receivers");
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in self.workers.drain(..) {
            println!("Shutting down worker {}", worker.id);
            worker.thread.join().unwrap();
        }
    }
}

struct Worker {
    id: usize,
    thread: JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, rx: Receiver<Job>) -> Self {
        let thread = thread::spawn(move || {
            loop {
                let worker_message = rx.recv();
                match worker_message {
                    Ok(job) => {
                        println!("Worker {id} got a job; executing.");
                        job();
                    }
                    Err(_) => {
                        println!("Worker {id} disconnected; shutting down.");
                        break;
                    }
                }
            }
        });
        Worker { id, thread }
    }
}
