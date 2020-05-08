use std::result::Result;
use std::thread::{JoinHandle, spawn};
use std::sync::{mpsc, Arc, Mutex};
use std::sync::mpsc::Receiver;

// TODO: Read chapter 16 of the book to understand the mpsc functions
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

struct Worker {
    id: usize,
    thread: JoinHandle<()>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = spawn( move || loop {
            // Panicing is the correct thing here.
            // > Here, we first call lock on the receiver to acquire the mutex,
            // > and then we call unwrap to panic on any errors.
            // > Acquiring a lock might fail if the mutex is in a poisoned state,
            // > which can happen if some other thread panicked while holding the lock
            // > rather than releasing the lock. In this situation, calling unwrap
            // > to have this thread panic is the correct action to take.
            // > Feel free to change this unwrap to an expect with an error message
            // > that is meaningful to you.
            // let job = receiver.lock().unwrap().recv().unwrap();
            match receiver.lock().unwrap().recv() {
                Ok(job) => {
                    println!("Worker {} got a job; executing.", id);

                    job();
                }
                Err(err) => {
                    println!("Received an error!");
                    println!("{}", err);
                }
            }

        });
        Worker { id, thread }
    }
}

#[derive(Debug)]
pub struct PoolCreationError(String);

impl ThreadPool {
    /// Create new ThreadPool
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Error
    ///
    /// The `new` function will return an Err of PoolCreationError
    /// which just contains a `String`.
    /// TODO: This could perhaps be a string literal.
    pub fn new(size: usize) ->
                            // Result<ThreadPool, PoolCreationError>
                            ThreadPool
    {
        // if size > 0 {
        //     let err_msg = "Cannot create pool with zero threads.".to_string();
        //     return Err(PoolCreationError(err_msg));
        // }
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        // This is slightly more efficient than `Vec::new`
        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        // Ok(ThreadPool { workers, sender })
        ThreadPool { workers, sender }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.send(job).unwrap();
    }
}