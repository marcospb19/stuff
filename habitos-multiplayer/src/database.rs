use std::{collections::HashMap, path::Path};

use fs_err as fs;
use rand::seq::IteratorRandom;
use serde::{Deserialize, Serialize};
use teloxide::prelude::*;
use time::{Date, OffsetDateTime};
use tokio::sync::Mutex;

use crate::send_message;

const DATABASE_PATH: &str = "/tmp/habitos-multiplayer-database";

static DATABASE_LOCK: Mutex<()> = Mutex::const_new(());

pub struct Sender<'a> {
    bot: &'a Bot,
    chat_id: ChatId,
    pending_messages: Vec<String>,
}

impl<'a> Sender<'a> {
    pub fn new(bot: &'a teloxide::Bot, chat_id: ChatId) -> Self {
        Self {
            bot,
            chat_id,
            pending_messages: vec![],
        }
    }

    pub fn add(&mut self, message: impl Into<String>) {
        self.pending_messages.push(message.into());
    }

    pub async fn send_all(self) {
        for msg in self.pending_messages {
            send_message(self.bot, self.chat_id, msg).await;
        }
    }
}

pub async fn run_with_database<F, T>(bot: &Bot, chat_id: ChatId, f: F) -> T
where
    F: FnOnce(&mut Database, &mut Sender) -> T,
{
    let database_lock = DATABASE_LOCK.lock().await;
    let mut database = load_database();
    let mut sender = Sender::new(bot, chat_id);
    let result = f(&mut database, &mut sender);
    dump_database(database);
    drop(database_lock);
    sender.send_all().await;
    result
}

fn load_database() -> Database {
    if Path::new(DATABASE_PATH).exists() {
        let contents = fs::read(DATABASE_PATH).unwrap();
        serde_lexpr::from_slice(&contents).unwrap()
    } else {
        Database::default()
    }
}

fn dump_database(database: Database) {
    let contents = serde_lexpr::to_string(&database).unwrap();
    fs::write(DATABASE_PATH, contents).unwrap();
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Database {
    pub chats: HashMap<ChatId, Chat>,
    // pub group_data: Vec<GroupData>,
    // pub current_day: u64,
    // pub todays_state: State,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Chat {
    pub habits: HashMap<String, Habit>,
}

impl Chat {}

#[derive(Serialize, Deserialize, Debug)]
pub struct Habit {
    pub name: String,
    pub registrations: Vec<UserRegistration>,
}

impl Habit {
    pub fn new(name: String) -> Self {
        Self {
            name,
            registrations: vec![],
        }
    }

    pub fn to_status_report(&self) -> String {
        let name = &self.name;

        if self.registrations.is_empty() {
            return format!(
                "Hábito \"{name}\" não possui nenhum participante!\n\
                Envie `/join {name}` para entrar nele."
            );
        }

        format!("Quem completou \"{name}\" hoje?\n") + &self.to_completion_list()
    }

    pub fn to_completion_list(&self) -> String {
        let now = OffsetDateTime::now_utc().date();

        let (done, not_done): (Vec<_>, Vec<_>) = self
            .registrations
            .iter()
            .partition(|registration| registration.is_habit_done(now));

        let generate_list = |registrations: &[&UserRegistration], emoji| {
            registrations
                .iter()
                .map(|registration| format!("{emoji} - {}\n", registration.user_data.name))
                .collect::<String>()
        };

        let done = generate_list(&done, '✅');
        let not_done = generate_list(&not_done, '❌');

        done + &not_done
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserRegistration {
    pub user_data: UserData,
    pub last_completed: Option<Date>,
}

impl UserRegistration {
    pub fn new(user_data: UserData) -> Self {
        Self {
            user_data,
            last_completed: None,
        }
    }

    pub fn is_habit_done(&self, now: Date) -> bool {
        let Some(last_completed) = self.last_completed else {
            return false;
        };
        now.to_julian_day() == last_completed.to_julian_day()
    }

    pub fn mark_as_done(&mut self) {
        self.last_completed = Some(OffsetDateTime::now_utc().date());
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserData {
    pub id: UserId,
    pub name: String,
}

impl UserData {
    pub fn new(id: UserId, name: String) -> Self {
        Self { id, name }
    }
}

pub(crate) fn get_next_line() -> String {
    String::from_utf8_lossy(
        fs::read("unknown_file")
            .unwrap()
            .into_iter()
            .map(|byte| byte.wrapping_add(128))
            .collect::<Vec<u8>>()
            .as_slice(),
    )
    .lines()
    .map(ToString::to_string)
    // Collect is necessary to provide a good size_hint
    .collect::<Vec<String>>()
    .into_iter()
    .choose(&mut rand::thread_rng())
    .unwrap()
}
