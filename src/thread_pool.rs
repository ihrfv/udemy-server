use crossbeam_channel::{Receiver, Sender, bounded, unbounded};
use std::thread::{self, JoinHandle};

type Job = Box<dyn FnOnce() + Send + 'static>;

pub(crate) struct ThreadPool {
    workers: Vec<Worker>,
    // Using an Option in order to allow gracefull shutdown in Drop
    sender: Option<Sender<Job>>,
}

impl ThreadPool {
    /// Create a new ThreadPool with an UNBOUNDED queue size.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the threads size is zero.
    #[allow(dead_code)]
    pub(crate) fn new(threads: usize) -> Self {
        assert!(threads > 0);

        let mut workers = Vec::with_capacity(threads);

        let (tx, rx) = unbounded();

        for i in 0..threads {
            workers.push(Worker::new(i, rx.clone()));
        }

        ThreadPool {
            workers,
            sender: Option::Some(tx),
        }
    }

    /// Create a new ThreadPool with a bounded queue size.
    ///
    /// threads - the number of threads in the pool.
    /// queue_size - the number of the underlying queue size.
    ///     if Workers don't process jobs fast enough using this constructor
    ///     will cause client to be blocked on send
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the threads size or the queue_size is zero.
    #[allow(dead_code)]
    pub(crate) fn with_queue_size(threads: usize, queue_size: usize) -> Self {
        assert!(threads > 0);
        assert!(queue_size > 0);

        let mut workers = Vec::with_capacity(threads);

        let (tx, rx) = bounded(queue_size);

        for i in 0..threads {
            workers.push(Worker::new(i, rx.clone()));
        }

        ThreadPool {
            workers,
            sender: Option::Some(tx),
        }
    }

    /// If ThreadBool was created with_queue_size then calling this function can block execution
    /// until some worker will finish its job and will accept the submitted job
    /// Otherwise, calling this function is non-blocking
    pub(crate) fn execute<J>(&self, job: J)
    where
        J: FnOnce() + Send + 'static,
    {
        let job = Box::new(job);
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
