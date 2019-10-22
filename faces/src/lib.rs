
mod sparse_distributed_representation;
pub mod udp;
//pub mod tcp;
pub use crate::{udp::Udp};//, tcp::Tcp};

use {
    packets::{Packet},
    futures::{
        future::{FutureExt, BoxFuture},
        task::{ArcWake, waker_ref},
    },
    std::{
        future::Future,
        sync::{Arc, Mutex},
        sync::mpsc::{sync_channel, SyncSender, Receiver},
        task::{Context, Poll},
    },
    crossbeam_channel::{
        unbounded, Sender
    },
};

pub trait Face {
    fn id(&self) -> u32;
    // router uses these
    fn send_interest_downstream(&mut self, interest: Packet);
    fn send_data_upstream(&mut self, data: Packet);

    fn create_pending_interest(&mut self, sdri: &Vec<Vec<u16>>);
    fn contains_pending_interest(&mut self, sdri: &Vec<Vec<u16>>) -> u8;
    fn delete_pending_interest(&mut self, sdri: &Vec<Vec<u16>>);
    fn pending_interest_decoherence(&mut self) -> u8;
    fn partially_forget_pending_interests(&mut self);

    fn create_forwarding_hint(&mut self, sdri: &Vec<Vec<u16>>);
    fn contains_forwarding_hint(&mut self, sdri: &Vec<Vec<u16>>) -> u8;
    fn forwarding_hint_decoherence(&mut self) -> u8;
    fn partially_forget_forwarding_hints(&mut self);

    // application uses these
    //fn try_interest(&self) -> Option<Packet>;
    //fn interest(&self) -> Option<Packet>;


    fn box_clone(&self) -> Box::<dyn Face>;
    fn receive_upstream_interest_or_downstream_data(&self, spawner: Spawner, packet_sender: Sender<Packet>);
    fn print_pi(&self);
    fn print_fh(&self);
}

impl Clone for Box<dyn Face> {
    fn clone(&self) -> Box<dyn Face> {
        self.box_clone()
    }
}

// the below is obtained from https://rust-lang.github.io/async-book/02_execution/04_executor.html see if it's possible to 1 not run the executor on a thread in the router (for WASM) and 2 use the generic upstream executor in futures, so that other applications can compose these async futures into their applications

pub struct Executor {
    ready_queue: Receiver<Arc<Task>>,
}

#[derive(Clone)]
pub struct Spawner {
    task_sender: SyncSender<Arc<Task>>,
}

pub struct Task {
    future: Mutex<Option<BoxFuture<'static, ()>>>,

    /// Handle to place the task itself back onto the task queue.
    task_sender: SyncSender<Arc<Task>>,
}

pub fn new_spawner_and_executor() -> (Spawner, Executor) {
    // Maximum number of tasks to allow queueing in the channel at once.
    // This is just to make `sync_channel` happy, and wouldn't be present in
    // a real executor.
    const MAX_QUEUED_TASKS: usize = 10_000;
    let (task_sender, ready_queue) = sync_channel(MAX_QUEUED_TASKS);
    (Spawner { task_sender}, Executor { ready_queue })
}

impl Spawner {
    fn spawn(&self, future: impl Future<Output = ()> + 'static + Send) {
        let future = future.boxed();
        let task = Arc::new(Task {
            future: Mutex::new(Some(future)),
            task_sender: self.task_sender.clone(),
        });
        self.task_sender.send(task).expect("too many tasks queued");
    }
}

impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        // Implement `wake` by sending this task back onto the task channel
        // so that it will be polled again by the executor.
        let cloned = arc_self.clone();
        arc_self.task_sender.send(cloned).expect("too many tasks queued");
    }
}

impl Executor {
    pub fn run(&self) {
        while let Ok(task) = self.ready_queue.recv() {
            // Take the future, and if it has not yet completed (is still Some),
            // poll it in an attempt to complete it.
            let mut future_slot = task.future.lock().unwrap();
            if let Some(mut future) = future_slot.take() {
                // Create a `LocalWaker` from the task itself
                let waker = waker_ref(&task);
                let context = &mut Context::from_waker(&*waker);
                // `BoxFuture<T>` is a type alias for
                // `Pin<Box<dyn Future<Output = T> + Send + 'static>>`.
                // We can get a `Pin<&mut dyn Future + Send + 'static>`
                // from it by calling the `Pin::as_mut` method.
                if let Poll::Pending = future.as_mut().poll(context) {
                    // We're not done processing the future, so put it
                    // back in its task to be run again in the future.
                    *future_slot = Some(future);
                }
            }
        }
    }
}
