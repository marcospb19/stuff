use std::{sync::mpsc, time::Instant};

/// Reimplement std::sync::mpsc::Receiver::recv_deadline in stable Rust.
pub fn recv_deadline<T>(receiver: &mpsc::Receiver<T>, instant: Instant) -> Result<T, mpsc::RecvTimeoutError> {
    let duration_to_sleep = instant.duration_since(Instant::now());
    receiver.recv_timeout(duration_to_sleep)
}
