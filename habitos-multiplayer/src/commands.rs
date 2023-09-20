use std::collections::hash_map::Entry;

use regex::{Captures, Regex};
use teloxide::{prelude::*, types::MessageKind};

use crate::{
    database::{get_next_line, run_with_database, Chat, Habit, UserData, UserRegistration},
    COMMANDS,
};

const ERROR_FALTOU_NOME_DO_HABITO: &str = "Erro: faltou prover o hábito que você quer adicionar. (/help)";

pub async fn start(bot: &Bot, command: Command<'_>) {
    bot.send_message(command.chat_id, "Fala tu! E roda um /help aí")
        .await
        .unwrap();
}

pub async fn help(bot: &Bot, command: Command<'_>) {
    let mut help_message = String::new();

    for (_, description) in COMMANDS {
        help_message += description;
        help_message += "\n";
    }

    bot.send_message(command.chat_id, help_message).await.unwrap();
}

pub async fn new(bot: &Bot, command: Command<'_>) {
    run_with_database(bot, command.chat_id, |db, sender| {
        let chat = db.chats.entry(command.chat_id).or_insert_with(Chat::default);

        let Some(habit_name) = command.habit_name else {
            sender.add(ERROR_FALTOU_NOME_DO_HABITO);
            return;
        };

        // Check if habit already exists
        match chat.habits.entry(habit_name.into()) {
            // If not, create a new habit
            Entry::Vacant(vacant) => {
                sender.add(format!(
                    "Novo hábito \"{habit_name}\" criado!\n\
                    Envie `/join {habit_name}` para entrar nele."
                ));
                vacant.insert(Habit::new(habit_name.into()));
            }
            // If so, tell that the habit already exists
            Entry::Occupied(habit) => {
                let response = format!("Hábito \"{habit_name}\" já existe!\n") + &habit.get().to_completion_list();
                sender.add(response);
            }
        }
    })
    .await;
}

pub async fn status(bot: &Bot, command: Command<'_>) {
    run_with_database(bot, command.chat_id, |db, sender| {
        let chat = db.chats.entry(command.chat_id).or_insert_with(Chat::default);

        let Some(habit_name) = command.habit_name else {
            sender.add(ERROR_FALTOU_NOME_DO_HABITO);
            return;
        };

        // Check if habit exists
        let Some(habit) = chat.habits.get_mut(habit_name) else {
            sender.add(habit_not_found(habit_name));
            return;
        };

        let response = habit.to_status_report();
        sender.add(response);
    })
    .await;
}

pub async fn join(bot: &Bot, command: Command<'_>) {
    run_with_database(bot, command.chat_id, |db, sender| {
        let chat = db.chats.entry(command.chat_id).or_insert_with(Chat::default);

        let Some(habit_name) = command.habit_name else {
            sender.add(ERROR_FALTOU_NOME_DO_HABITO);
            return;
        };

        // Check if habit exists
        let Some(habit) = chat.habits.get_mut(habit_name) else {
            sender.add(habit_not_found(habit_name));
            return;
        };

        let message_end = format!(
            "\nRode `/status {habit_name}` para ver como está.\
             \nRode `/done {habit_name}` para marcar como feito!"
        );

        let user = command.user;

        match habit
            .registrations
            .iter()
            .find(|registration| registration.user_data.id == user.id)
        {
            Some(_) => sender.add(format!(
                "{} já está registrado em {habit_name}!{message_end}",
                user.name
            )),
            None => {
                sender.add(format!("{} adicionado á {habit_name}!{message_end}", user.name));
                habit.registrations.push(UserRegistration::new(user));
            }
        }
    })
    .await;
}

pub async fn done(bot: &Bot, command: Command<'_>) {
    run_with_database(bot, command.chat_id, |db, sender| {
        let chat = db.chats.entry(command.chat_id).or_insert_with(Chat::default);

        let Some(habit_name) = command.habit_name else {
            sender.add(ERROR_FALTOU_NOME_DO_HABITO);
            return;
        };

        // Check if habit exists
        let Some(habit) = chat.habits.get_mut(habit_name) else {
            sender.add(habit_not_found(habit_name));
            return;
        };

        let user = command.user;

        match habit
            .registrations
            .iter_mut()
            .find(|registration| registration.user_data.id == user.id)
        {
            Some(user_registration) => {
                user_registration.mark_as_done();
                let line = get_next_line();

                sender.add(format!("{line}\n{}", habit.to_completion_list()));
            }
            None => {
                sender.add(format!(
                    "Calma lá! 👀 Tem que entrar no hábito para poder marcar quando fizer ele.\n\
                    Envie `/join {habit_name}` para entrar 👍."
                ));
            }
        }
    })
    .await;
}

fn habit_not_found(habit_name: &str) -> String {
    format!(
        "Erro: Hábito \"{habit_name}\" não encontrado!\n\
        Envie `/new {habit_name}` para criar ele."
    )
}

// A telegram line of command
pub struct Command<'a> {
    pub trimmed: &'a str,
    pub short_slash: &'a str,
    pub bot_mention: Option<&'a str>,
    pub habit_name: Option<&'a str>,
    pub chat_id: ChatId,
    pub user: UserData,
}

impl<'a> Command<'a> {
    pub fn from_message(msg: &'a Message) -> Result<Self, &'static str> {
        let text = msg.text().unwrap();
        let chat_id = msg.chat.id;
        let user = {
            let common_message = match &msg.kind {
                MessageKind::Common(inner) => Some(inner),
                _ => None,
            }
            .ok_or_else(|| {
                eprintln!("Message type: {:?}", msg.kind);
                "Message expected to be of kind 'common', found another one"
            })?;

            let user = common_message
                .from
                .as_ref()
                .ok_or("Could not read user_id for this messageis it a common message?")?;

            UserData::new(user.id, user.first_name.clone())
        };

        let trimmed = text.trim();
        if trimmed.contains('\n') {
            return Err("O comando deveria caber em uma linha só, você escreveu múltiplas linhas");
        }

        let pattern = r"^(?<short_slash>/[^\s@]+)(?<bot_mention>@\S*)?(\s+(?<habit_name>\S+))?.*$";
        let regex = Regex::new(pattern).unwrap();

        let captures: Captures<'a> = regex
            .captures(trimmed)
            .ok_or("Falha interna no bot, comando não pode ser lido...")?;

        let short_slash = captures.name("short_slash").unwrap().as_str();
        let bot_mention = captures.name("bot_mention").map(|match_| match_.as_str());
        let habit_name = captures.name("habit_name").map(|match_| match_.as_str());

        Ok(Self {
            trimmed,
            short_slash,
            habit_name,
            bot_mention,
            chat_id,
            user,
        })
    }
}
