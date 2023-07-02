mod colors;
mod error;
mod notification;
mod stdin;

use std::{
    sync::mpsc::Receiver,
    time::{Duration, Instant},
};

use clap::Parser;
use owo_colors::OwoColorize;

use crate::{notification::send_notification, stdin::spawn_stdin_channel};

#[derive(Parser)]
enum CliArgs {
    /// A standard sprint of 4 x 25/5.
    Sprint,
    /// Run tomates of custom Work/Rest.
    Custom { work_time: u64, rest_time: u64 },
}

fn main() {
    let args = CliArgs::parse();

    let stdin_rx = spawn_stdin_channel();
    let tomato = Tomato::new(stdin_rx);

    match args {
        CliArgs::Sprint => tomato.run_sprint(),
        CliArgs::Custom { work_time, rest_time } => tomato.work_time(work_time).rest_time(rest_time).run_indefinetely(),
    }
}

struct Tomato {
    work_time: u64,
    rest_time: u64,
    current_tomato: u64,
    stdin_receiver: Receiver<String>,
    reward_emoji_iter: Box<dyn Iterator<Item = &'static str>>,
    micro_management_emoji_iter: Box<dyn Iterator<Item = &'static str>>,
}

impl Tomato {
    pub fn new(stdin_receiver: Receiver<String>) -> Self {
        Self {
            work_time: 25,
            rest_time: 5,
            current_tomato: 0,
            stdin_receiver,
            reward_emoji_iter: Box::new(["🍅", "🥗", "🍝", "🍕"].into_iter().cycle()),
            micro_management_emoji_iter: Box::new(["👀", "🔫", "👮", "🚨"].into_iter().cycle()),
        }
    }

    pub fn work_time(self, work_time: u64) -> Self {
        Self { work_time, ..self }
    }

    pub fn rest_time(self, rest_time: u64) -> Self {
        Self { rest_time, ..self }
    }

    pub fn run_sprint(mut self) {
        while self.current_tomato < 5 {
            self.run_once();
        }
    }

    pub fn run_indefinetely(mut self) {
        loop {
            self.run_once();
        }
    }

    fn run_once(&mut self) {
        self.current_tomato += 1;

        self.run_work_timer();
        self.run_rest_timer();
    }

    fn run_work_timer(&mut self) {
        showln!(
            format_args!("[{}]", self.current_tomato).magenta(),
            " Tomate de ",
            format_args!("{} minutos", self.work_time).blue(),
            " iniciado! ",
        );

        let total_duration = Duration::from_secs(self.work_time * 60);
        let half_duration = total_duration / 2;

        let was_skipped = self.run_pausable_timer(half_duration);

        // Extra logic to be able to send a notification at the half
        if !was_skipped {
            send_notification(format!(
                "Na metade! Você está focado não está? {}",
                self.micro_management_emoji_iter.next().unwrap(),
            ));
            self.run_pausable_timer(half_duration);
        }

        showln!(
            "Tomate ",
            format_args!("{}", self.current_tomato).red(),
            " concluído!".green(),
            " Sua recompensa: ",
            self.reward_emoji_iter.next().unwrap(),
        );
    }

    fn run_rest_timer(&mut self) {
        let total_duration = Duration::from_secs(self.rest_time * 60);
        self.run_pausable_timer(total_duration);
    }

    /// Returns if timer was skipped.
    fn run_pausable_timer(&mut self, mut remaining: Duration) -> bool {
        loop {
            let start_instant = Instant::now();

            // // // How can I show this?
            // // Work for 24:59 (25 minutes)
            // println!("\r"); // !?!?!?!?

            match self.sleep_for(remaining) {
                SleepResult::Finished => return false,
                SleepResult::Skipped => return true,
                SleepResult::Paused => { /* continue and handle pause */ }
            }

            let elapsed = start_instant.elapsed();
            remaining = remaining.saturating_sub(elapsed);
            showln!("Paused.".red());

            if self.wait_unpause() {
                break true;
            }

            let _ = self.stdin_receiver.recv();
            showln!("Unpaused.".green());
        }
    }

    fn sleep_for(&self, duration: Duration) -> SleepResult {
        match self.stdin_receiver.recv_timeout(duration) {
            Ok(line) if line.contains("skip") => SleepResult::Skipped,
            Ok(_) => SleepResult::Paused,
            Err(_) => SleepResult::Finished,
        }
    }

    /// Returns if skipping was requested
    fn wait_unpause(&self) -> bool {
        self.stdin_receiver.recv().unwrap().contains("skip")
    }
}

enum SleepResult {
    Finished,
    Paused,
    Skipped,
}
