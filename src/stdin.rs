use std::{
    io::{self, BufRead},
    mem,
    sync::mpsc::{self, Receiver, SyncSender},
    thread,
};

use crate::error::UnwrapOrExplode;

pub fn spawn_stdin_channel() -> Receiver<String> {
    // Create a Rendezvous Channel (backpressure with N = 0)
    let (stdin_tx, stdin_rx) = mpsc::sync_channel::<String>(0);

    // For convenience on the receiver side, turn the channel impossible to close.
    mem::forget(stdin_tx.clone());

    thread::spawn(|| run_stdin_reader(stdin_tx));

    stdin_rx
}

fn run_stdin_reader(sender: SyncSender<String>) {
    let mut stdin = io::stdin().lock();

    loop {
        let mut string = String::new();

        match stdin.read_line(&mut string) {
            Ok(0) | Err(_) => return,
            _ => {}
        }

        sender.send(string).unwrap_or_explode("Internal STDIN sender failed");
    }
}
