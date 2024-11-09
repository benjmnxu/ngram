use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

// We represent a job as a boxed closure that can be sent across threads. Since the closure is
// `Send`, it can be sent across threads. Since it is in a box, we have ownership and can transfer
// it to other threads.
type Job = Box<dyn FnOnce() + Send + 'static>;

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}
impl Worker {
    // TODO:
    // Spawn a new thread that will loop forever, receiving jobs from the receiver and executing
    // them. If the `recv()` method returns an error, it means the thread pool has been dropped and
    // the thread should exit by breaking the loop.
    // This function should return a `Worker` as a handle to the thread.
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || {
            loop {
                let job = receiver.lock().unwrap().recv();

                match job {
                    Ok(job) => {
                        println!("Worker {} got a job; executing.", id);
                        job(); // Execute the job
                    }
                    Err(_) => {
                        println!("Worker {} disconnected; exiting.", id);
                        break;
                    }
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
    // TODO:
    // Spawn `size` workers by calling the `Worker::new` function `size` times, each time with a
    // unique id. You will need to create a channel and wrap the receiver in an `Arc<Mutex<...>>`
    // in order to share it with the worker threads. Finally, return an instance of `ThreadPool`
    // that has the workers and the sender.
    pub fn new(size: usize) -> ThreadPool {
        let mut workers = Vec::new();
        let (tx, rx) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(rx));
        for i in 0..size {
            workers.push(Worker::new(i, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            sender: Some(tx),
        }
    }

    // TODO:
    // Send the job `f` to the worker threads via the channel `send` method.
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        if let Some(ref sender) = self.sender {
            // Wrap the closure in a Box and send it through the channel
            if let Err(err) = sender.send(Box::new(f)) {
                eprintln!("Failed to send job to worker: {}", err);
            }
        } else {
            eprintln!("Thread pool has no active sender.");
        }
    }
}

impl Drop for ThreadPool {
    // TODO:
    // First, take ownership of the sender from inside the option, then drop it. This will trigger
    // the worker threads to stop since the channel is closed, so you should then call `join` on
    // each worker thread handle to make sure they finish executing. Calling `join` will also
    // require you to take ownership of the worker thread handle from inside the option.
    fn drop(&mut self) {
        self.sender = None;
        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                thread.join().expect("");
            }
        }
    }
}
