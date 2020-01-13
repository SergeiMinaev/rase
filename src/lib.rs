use std::thread;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;

pub mod config_parser;
pub mod logger;
pub mod mime;

pub struct ThreadPool {
	workers: Vec<Worker>,
	sender: mpsc::SyncSender<Message>,
}
trait FnBox {
	fn call_box(self: Box<Self>);
}
impl<F: FnOnce()> FnBox for F {
	fn call_box(self: Box<F>) {
		(*self)();
	}
}
type Job = Box<dyn FnBox + Send + 'static>;

enum Message {
	NewJob(Job),
	Terminate,
}
struct Worker {
	id: usize,
	thread: Option<thread::JoinHandle<()>>,
}


impl ThreadPool {
	pub fn new(thread_count: usize) -> ThreadPool {

        if thread_count <= 0 {
            panic!("Number of threads should be bigger than 0.");
        }

		let (sender, receiver) = mpsc::sync_channel(1000);
		let receiver = Arc::new(Mutex::new(receiver));
		let mut workers = Vec::with_capacity(thread_count);
		for id in 0..thread_count {
			workers.push(Worker::new(id, Arc::clone(&receiver)));
		}
		ThreadPool {
			workers,
			sender,
		}
	}

	pub fn execute<F>(&self, f: F)
		where
			F: FnOnce() + Send + 'static
	{
        //println!("execute()");
		let job = Box::new(f);
		//self.sender.try_send(Message::NewJob(job)).unwrap();
		let r = self.sender.try_send(Message::NewJob(job));
        if r.is_err() {
            println!("R: {:?}", r);
        }

	}

    pub fn kill_some_workers(count: usize) {
        println!("Killing workers: {}", count);
    }
}

impl Drop for ThreadPool {
	fn drop(&mut self) {
		println!("Sending terminate to all workers");
		for _ in &mut self.workers {
			self.sender.send(Message::Terminate).unwrap();
		}
		println!("Shutting down all workers");
		for worker in &mut self.workers {
			println!("Shutting down worker {}", worker.id);

			if let Some(thread) = worker.thread.take() {
				thread.join().unwrap();
			}
		}
	}
}

impl Worker {
	fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
		let thread = thread::spawn(move || {
			loop {
                //println!("Loop {}", id);
				let message = receiver.lock().unwrap().recv().unwrap();
				match message {
					Message::NewJob(job) => {
						//println!("Worker {} got a job; executing.", id);
						job.call_box();
					},
					Message::Terminate => {
						println!("Worker {} was told to terminate", id);
						break;
					},
				}
			}
		});

		Worker {
			id,
			thread: Some(thread),
		}
	}

}
