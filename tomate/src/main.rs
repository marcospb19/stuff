mod bar_integration;
mod colors;
mod error;
mod nightly;
mod notification;
mod stdin;
mod time;

const CLEAR_LINE: &str = "\x1B[2K";

use std::{
    fmt, io,
    io::Write,
    sync::mpsc::Receiver,
    time::{Duration, Instant},
};

use clap::Parser;
use owo_colors::OwoColorize;

use crate::{
    bar_integration::{BarMessage, BarMessager},
    error::UnwrapOrExplode,
    nightly::recv_deadline,
    notification::send_notification,
    stdin::spawn_stdin_channel,
    time::Time,
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
    bar_messager: BarMessager,
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
            bar_messager: BarMessager::new().unwrap_or_explode("Failed to connect bar messager"),
        }
    }

    pub fn set_work_time(self, work_time: u32) -> Self {
        (work_time != 0).unwrap_or_explode("the work_time argument can't be zero!");

        (work_time < 60).unwrap_or_explode("the work_time argument cannot be bigger than a hour.");

        Self { work_time, ..self }
    }

    pub fn set_rest_time(self, rest_time: u32) -> Self {
        (rest_time != 0).unwrap_or_explode("the rest_time argument can't be zero!");

        (rest_time < 60).unwrap_or_explode("the rest_time argument cannot be bigger than a hour.");

        Self { rest_time, ..self }
    }

    pub fn run_sprint(mut self) {
        while self.current_tomato < 4 {
            self.run_once();
        }

        self.bar_messager.send_message(BarMessage::Disconnecting).unwrap();
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

        let was_skipped = self.run_pausable_timer(half_duration, half_duration, Stage::Work);

        // Extra logic to be able to send a notification at the half
        if !was_skipped {
            send_notification(format!(
                "Na metade! Você está focado, não está? {}",
                self.micro_management_emoji_iter.next().unwrap(),
            ));
            self.run_pausable_timer(half_duration, None, Stage::Work);
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
        self.run_pausable_timer(total_duration, None, Stage::Rest);
        println!();
    }

    fn run_pausable_timer(
        &mut self,
        mut remaining: Duration,
        additional_time_to_display: impl Into<Option<Duration>>,
        status: Stage,
    ) -> bool {
        let additional_time_to_display = additional_time_to_display.into().unwrap_or_default();
        let start_instant = Instant::now();

        let increment = Duration::from_secs(1);
        let mut increment_sum = increment;

        let mut stdout = io::stdout();

        while remaining != Duration::ZERO {
            let time = Time::from(remaining + additional_time_to_display);

            // Print line
            write!(stdout, "{CLEAR_LINE}\r  {status} {time}          ").unwrap();
            stdout.flush().unwrap();
            self.bar_messager
                .send_message(BarMessage::Running(time, status))
                .unwrap();

            // Sleep
            let instant_to_reach = start_instant + increment_sum;

            if let Ok(line) = recv_deadline(&self.stdin_receiver, instant_to_reach) {
                go_back_one_line();

                if !line.contains('p') {
                    write!(stdout, "{CLEAR_LINE}\r  {status} {time} {} ", "(Paused)".red()).unwrap();
                    stdout.flush().unwrap();
                    self.bar_messager
                        .send_message(BarMessage::Paused(time, status))
                        .unwrap();

                    if !self.wait_unpause() {
                        return self.run_pausable_timer(remaining, additional_time_to_display, status);
                    }
                }

                write!(stdout, "{CLEAR_LINE}\r  {status} {} at {time}.", "skipped".red()).unwrap();
                return true;
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

#[derive(Clone, Copy)]
pub enum Stage {
    Work,
    Rest,
}

impl fmt::Display for Stage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Work => write!(f, "{}", "[Work]".magenta()),
            Self::Rest => write!(f, "{}", "[Rest]".cyan()),
        }
    }
}
