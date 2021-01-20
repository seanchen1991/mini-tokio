mod task;

use task::Task;

use crossbeam::channel;

use std::future::Future;
use std::sync::Arc;

pub struct MiniTokio {
    scheduled: channel::Receiver<Arc<Task>>,
    sender: channel::Sender<Arc<Task>>,
}

impl Default for MiniTokio {
    fn default() -> Self {
        let (sender, scheduled) = channel::unbounded();
        MiniTokio { scheduled, sender }
    }
}

impl MiniTokio {
    /// Spawn a future onto the mini-tokio instance.
    pub fn spawn<F>(&mut self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        Task::spawn(future, &self.sender);
    }

    /// Run through all tasks in the task queue and drive them
    /// to completion.
    pub fn run(&mut self) {
        while let Ok(task) = self.scheduled.recv() {
            task.poll();
        }
    }
}
