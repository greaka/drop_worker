#[cfg(feature = "crossbeam")]
use crossbeam::{unbounded as channel, Sender};
#[cfg(feature = "crossbeam")]
pub use crossbeam::{Receiver, TryRecvError};
#[cfg(not(feature = "crossbeam"))]
use std::sync::mpsc::{channel, Sender};
#[cfg(not(feature = "crossbeam"))]
pub use std::sync::mpsc::{Receiver, TryRecvError};

use std::{mem::ManuallyDrop, ops::Deref, thread::JoinHandle};

pub struct DropWorker<T> {
    sender: ManuallyDrop<Sender<T>>,
    thread: JoinHandle<()>,
}

#[macro_export]
macro_rules! try_err {
    ($recv:expr) => {
        if let Err(drop_worker::TryRecvError::Disconnected) = $recv.try_recv() {
            return;
        }
    };
}

#[macro_export]
macro_rules! recv_data {
    ($recv:expr) => {
        if let Ok(data) = $recv.recv() {
            data
        } else {
            return;
        };
    };
}

impl<T: Send + 'static> DropWorker<T> {
    pub fn new<F: Fn(Receiver<T>) + Send + 'static>(func: F) -> Self {
        let (sender, receiver) = channel::<T>();
        let sender = ManuallyDrop::new(sender);
        let thread = std::thread::spawn(move || func(receiver));
        DropWorker { sender, thread }
    }
}

impl<T: Send + 'static> Deref for DropWorker<T> {
    type Target = ManuallyDrop<Sender<T>>;

    fn deref(&self) -> &Self::Target {
        &self.sender
    }
}

impl<T> Drop for DropWorker<T> {
    fn drop(&mut self) {
        let thread;
        unsafe {
            ManuallyDrop::drop(&mut self.sender);
            thread = std::mem::replace(&mut self.thread, std::thread::spawn(|| ()));
        }
        let _ = thread.join();
    }
}
