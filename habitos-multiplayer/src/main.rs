mod commands;
mod database;

use std::future::Future;

use teloxide::{
    prelude::*,
    types::{AllowedUpdate, BotCommand, ParseMode},
    update_listeners::Polling,
    RequestError,
};
use tokio::signal;

use crate::commands::Command;

type Result<T> = std::result::Result<T, RequestError>;

// Ayy

const BOT_ARROBA: &str = "@habitos_multiplayer_bot";

const COMMANDS: &[(&str, &str)] = [
    ("/start", "'/start' - Mandar um oi pra mim"),
    // TODO: MELHORAR /HELP PRA EXPLICAR DE FATO O QUE O BOT FAZ, E COMO EU DEVO COMEÇAR USANDO ELE
    ("/help", "'/help' - Ver mensagem de ajuda"),
    ("/new", "'/new <HABITO>' - Para iniciar um novo hábito nesse grupo"),
    ("/join", "'/join <HABITO>' - Para entrar num hábito"),
    ("/done", "'/done <HABITO>' - Para marcar como feito"),
    ("/status", "'/status <HABITO>' - Para dar os detalhes de um hábito"),
    ("/list", "'/list' - Liste todos hábitos do grupo (TODO)"), // TODO
    (
        "/delete",
        "'/delete <HABITO>' - Para deletar um hábito nesse grupo (TODO)",
    ), // TODO
    ("/leave", "'/leave <HABITO>' - Para sair de um hábito (TODO)"), // TODO
]
.as_slice();

#[tokio::main]
async fn main() {
    make_interruptible(run()).await;
}

async fn run() {
    let token = include_str!("../token.txt");

    let bot = Bot::new(token);

    perform_setup(&bot).await;

    let listener = {
        Polling::builder(bot.clone())
            .allowed_updates(vec![AllowedUpdate::Message])
            .build()
    };

    teloxide::repl_with_listener(bot, handler, listener).await;
}

async fn handler(bot: Bot, msg: Message) -> Result<()> {
    let Some(text) = msg.text() else { return Ok(()) };

    if !text.trim().starts_with('/') {
        // Might be just a reply to the bot message
        return Ok(());
    }

    match Command::from_message(&msg) {
        Ok(command) => handle_command(&bot, command).await,
        Err(err) => eprintln!("Erro: {err}"),
    }

    Ok(())
}

async fn handle_command(bot: &Bot, command: Command<'_>) {
    match dbg!(command.short_slash) {
        "/start" => commands::start(bot, command).await,
        "/help" => commands::help(bot, command).await,
        "/new" => commands::new(bot, command).await,
        // "/delete" => commands::delete(bot, command).await,
        "/done" => commands::done(bot, command).await,
        "/join" => commands::join(bot, command).await,
        // "/leave" => commands::leave(bot, command).await,
        // "/list" => commands::list(bot, command).await,
        "/status" => commands::status(bot, command).await,
        _ => {
            if command.bot_mention == Some(BOT_ARROBA) {
                let msg = "Comando não reconhecido, veja comandos disponíveis com `/help`";
                send_message(bot, command.chat_id, msg).await;
            }
        }
    }
}

async fn perform_setup(bot: &Bot) {
    // Show commands previews to users
    let commands = COMMANDS.iter().map(|(command, description)| BotCommand {
        command: command.to_string(),
        description: description.to_string(),
    });

    bot.set_my_commands(commands).await.unwrap();
}

async fn send_message(bot: &Bot, chat_id: ChatId, message: impl Into<String>) {
    #[allow(deprecated)]
    let bot = bot.parse_mode(ParseMode::Markdown);

    bot.send_message(chat_id, message).await.unwrap();
}

pub async fn make_interruptible(f: impl Future) {
    tokio::select! {
        _ = f => (),
        _ = signal::ctrl_c() => (),
    }
}
