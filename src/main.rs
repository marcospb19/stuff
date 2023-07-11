mod colors;
mod error;
mod nightly;
mod notification;
mod stdin;

use std::{
    fmt::Display,
    io,
    io::Write,
    sync::mpsc::Receiver,
    time::{Duration, Instant},
};

use clap::Parser;
use owo_colors::OwoColorize;

use crate::{
    error::UnwrapOrExplode, nightly::recv_deadline, notification::send_notification, stdin::spawn_stdin_channel,
};

const MINUTE: Duration = Duration::from_secs(60);

#[derive(Parser)]
struct CliArgs {
    work_time: Option<u32>,
    rest_time: Option<u32>,
}

fn main() {
    let args = CliArgs::parse();

    let mut tomato = Tomato::new();

    if let Some(work) = args.work_time {
        tomato = tomato.set_work_time(work);
    }
    if let Some(rest) = args.rest_time {
        tomato = tomato.set_rest_time(rest);
    }

    tomato.run_sprint();
}

struct Tomato {
    work_time: u32,
    rest_time: u32,
    current_tomato: u64,
    stdin_receiver: Receiver<String>,
    reward_emoji_iter: Box<dyn Iterator<Item = &'static str>>,
    micro_management_emoji_iter: Box<dyn Iterator<Item = &'static str>>,
}

impl Tomato {
    pub fn new() -> Self {
        Self {
            work_time: 25,
            rest_time: 5,
            current_tomato: 0,
            stdin_receiver: spawn_stdin_channel(),
            reward_emoji_iter: Box::new(["🍅", "🥗", "🍝", "🍕"].into_iter().cycle()),
            micro_management_emoji_iter: Box::new(["👀", "🔫", "👮", "🚨"].into_iter().cycle()),
        }
    }

    pub fn set_work_time(self, work_time: u32) -> Self {
        (work_time != 0)
            .then_some(())
            .unwrap_or_explode("the work_time argument can't be zero!");
        Self { work_time, ..self }
    }

    pub fn set_rest_time(self, rest_time: u32) -> Self {
        (rest_time != 0)
            .then_some(())
            .unwrap_or_explode("the rest_time argument can't be zero!");
        Self { rest_time, ..self }
    }

    pub fn run_sprint(mut self) {
        while self.current_tomato < 4 {
            self.run_once();
        }
    }

    fn run_once(&mut self) {
        self.current_tomato += 1;

        self.run_work_timer();
        self.run_rest_timer();
    }

    fn run_work_timer(&mut self) {
        send_notification(format!("Iniciando tomate de {} minutos!", self.work_time));
        showln!(
            format_args!("[{}]", self.current_tomato).red(),
            " Tomate de ",
            format_args!("{} minutos", self.work_time).blue(),
            " iniciado! ",
        );

        let total_duration = MINUTE * self.work_time;
        let half_duration = total_duration / 2;

        let status = "[Work]".magenta();
        let was_skipped = self.run_pausable_timer(half_duration, half_duration, &status);

        // Extra logic to be able to send a notification at the half
        if !was_skipped {
            send_notification(format!(
                "Na metade! Você está focado, não está? {}",
                self.micro_management_emoji_iter.next().unwrap(),
            ));
            self.run_pausable_timer(half_duration, None, &status);
        }

        let reward_emoji = self.reward_emoji_iter.next().unwrap();

        showln!(
            "\n  ",
            "[Eba!]".green(),
            " Tomate ",
            format_args!("{}", self.current_tomato).red(),
            " concluído!".green(),
            " Sua recompensa: ",
            reward_emoji,
        );

        send_notification(format!(
            "Tomate {} concluído! {reward_emoji} Descanse {} minutos.",
            self.current_tomato, self.rest_time,
        ));
    }

    fn run_rest_timer(&mut self) {
        let total_duration = MINUTE * self.rest_time;
        self.run_pausable_timer(total_duration, None, "[Rest]".cyan());
        println!();
    }

    fn run_pausable_timer(
        &self,
        mut remaining: Duration,
        additional_time_to_display: impl Into<Option<Duration>>,
        status: impl Display,
    ) -> bool {
        let additional_time_to_display = additional_time_to_display.into().unwrap_or_default();
        let start_instant = Instant::now();

        let increment = Duration::from_secs(1);
        let mut increment_sum = increment;

        let mut stdout = io::stdout();

        while remaining != Duration::ZERO {
            // Print line
            let display_remaining = remaining + additional_time_to_display;
            let remaining_minutes = display_remaining.as_secs() / 60;
            let remaining_seconds = display_remaining.as_secs() % 60;
            write!(
                stdout,
                "\r  {status} {remaining_minutes:02}:{remaining_seconds:02}          ",
            )
            .unwrap();
            stdout.flush().unwrap();

            // Sleep
            let instant_to_reach = start_instant + increment_sum;

            if let Ok(line) = recv_deadline(&self.stdin_receiver, instant_to_reach) {
                go_back_one_line();

                if line.contains('p') {
                    return true;
                } else {
                    write!(
                        stdout,
                        "\r  {status} {remaining_minutes:02}:{remaining_seconds:02} {} ",
                        "(Paused)".red()
                    )
                    .unwrap();
                    stdout.flush().unwrap();

                    return if self.wait_unpause() {
                        true
                    } else {
                        self.run_pausable_timer(remaining, additional_time_to_display, status)
                    };
                }
            }

            // Account for slept duration
            increment_sum += increment;
            remaining -= increment;
        }
        false
    }

    /// Returns if skipping was requested
    fn wait_unpause(&self) -> bool {
        let was_skipped = self.stdin_receiver.recv().unwrap().contains('p');
        go_back_one_line();
        was_skipped
    }
}

fn go_back_one_line() {
    print!("\x1B[1A\x1B[{}D", u16::MAX);
}
