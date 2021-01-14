//! Provides helpful worker threads that get signaled to stop when dropped.
//!
//! # Features
//! The `crossbeam` feature will use unbounded [crossbeam](https://crates.io/crates/crossbeam) channels instead of [std](std::sync::mpsc) channels.
//!
//! # Example
//! ```
//! #[macro_use]
//! extern crate drop_worker;
//!
//! use drop_worker::{recv_data, try_err, DropWorker, Receiver};
//!
//! fn main() {
//!     let _worker = DropWorker::new(work);
//!
//!     let mut receiver = DropWorker::new(rec);
//!     receiver.send(5);
//! }
//!
//! fn work(recv: Receiver<()>) {
//!     // setup
//!     loop {
//!         try_err!(recv);
//!         // do work
//!     }
//! }
//!
//! fn rec(recv: Receiver<usize>) {
//!     loop {
//!         let data = recv_data!(recv);
//!         assert_eq!(data, 5);
//!     }
//! }
//! ```

#[cfg(feature = "crossbeam")]
use crossbeam::{unbounded as channel, Sender};
#[cfg(feature = "crossbeam")]
pub use crossbeam::{Receiver, TryRecvError};
#[cfg(not(feature = "crossbeam"))]
use std::sync::mpsc::{channel, Sender};
#[cfg(not(feature = "crossbeam"))]
pub use std::sync::mpsc::{Receiver, TryRecvError};

use std::{
	mem::ManuallyDrop, 
	ops::Deref, 
//	thread::JoinHandle
};

/// Provides a worker thread that can receive structs of type `T`.
/// When this instance is dropped, it will signal the worker thread to stop.
pub struct DropWorker<T> {
    sender: ManuallyDrop<Sender<T>>,
//    thread: JoinHandle<()>,
}

/// Checks if the [`DropWorker`] was dropped and if so then this will call
/// `return`.
#[macro_export]
macro_rules! try_err {
    ($recv:expr) => {
        if let Err(drop_worker::TryRecvError::Disconnected) = $recv.try_recv() {
            return;
        }
    };
}

/// Waits for data from the [`DropWorker`] and returns the data.
/// If the [`DropWorker`] gets dropped then this will call `return`.
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
    /// Spawns a new worker thread with the given function.
    /// The function must accept a [`Receiver`].
    pub fn new<F: Fn(Receiver<T>) + Send + 'static>(func: F) -> Self {
        let (sender, receiver) = channel::<T>();
        let sender = ManuallyDrop::new(sender);
        let _thread = std::thread::spawn(move || func(receiver));
        DropWorker { 
        	sender, 
//        	thread 
        }
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
        // let thread;
        unsafe {
            ManuallyDrop::drop(&mut self.sender);
            // thread = std::mem::replace(&mut self.thread, std::thread::spawn(|| ()));
        }
        // let _ = thread.join();
    }
}
