#![allow(unused_imports)]
#![allow(dead_code)]
/*!
Re-exports of some shared types that are used by multiple mail services.
*/
pub use flume::{Receiver, Sender};
pub use parking_lot::Mutex;
/// Create an unbounded channel.
pub fn unbdounded_channel<T>() -> (Sender<T>, Receiver<T>) {
    flume::unbounded()
}
/// Create a bounded channel with the given size.
pub fn bounded_channel<T>(size: usize) -> (Sender<T>, Receiver<T>) {
    flume::bounded(size)
}
