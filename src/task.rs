use crossbeam::channel;
use futures::task::{self, ArcWake};

use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::Context;

pub struct Task {
    // The Mutex is to make `Task` implement `Sync`. Only
    // one thread access `future` at any given time. The
    // Mutex is not required for correctness.
    future: Mutex<Pin<Box<dyn Future<Output = ()> + Send>>>,
    executor: channel::Sender<Arc<Task>>,
}

impl Task {
    fn schedule(self: &Arc<Self>) {
        // Increment the `Arc` and send it down the
        // executor channel
        self.executor.send(self.clone());
    }

    pub fn poll(self: Arc<Self>) {
        // Create a waker from the `Task` instance. This
        // uses the `ArcWake` impl.
        let waker = task::waker(self.clone());
        let mut cx = Context::from_waker(&waker);

        // No other thread ever tries to lock the future
        let mut future = self.future.try_lock().unwrap();

        let _ = future.as_mut().poll(&mut cx);
    }

    /// Spawns a new task with the given future.
    ///
    /// Initializes a new Task harness containing the given future
    /// and pushes it onto `sender`. The receiver half of the
    /// channel will get the task and execute it.
    pub fn spawn<F>(future: F, sender: &channel::Sender<Arc<Task>>)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let task = Arc::new(Task {
            future: Mutex::new(Box::pin(future)),
            executor: sender.clone(),
        });

        let _ = sender.send(task);
    }
}

impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        arc_self.schedule();
    }
}
