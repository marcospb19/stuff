use std::io;

use crate::{Stage, Time};

#[allow(unused)]
const CLIMSG_CHANNEL: &str = "tomate-pomodoro";

pub enum BarMessage {
    Running(Time, Stage),
    Paused(Time, Stage),
    Disconnecting,
}

cfg_if::cfg_if! {
    if #[cfg(feature = "bar-integration")] {
        use climsg_core::{ClientMessage, MessageStream};

        pub struct BarMessager {
            stream: MessageStream,
        }

        impl BarMessager {
            pub fn new() -> io::Result<Self> {
                MessageStream::connect_to_default().map(|stream| Self { stream })
            }

            pub fn send_message(&mut self, message: BarMessage) -> climsg_core::Result<()> {
                let msg = match message {
                    BarMessage::Running(time, Stage::Work) => format!(" work - {time} "),
                    BarMessage::Running(time, Stage::Rest) => format!(" rest - {time} "),
                    BarMessage::Paused(time, Stage::Work) => format!(" work - {time} (Paused) "),
                    BarMessage::Paused(time, Stage::Rest) => format!(" rest - {time} (Paused) "),
                    BarMessage::Disconnecting => String::new(),
                };

                self.stream.send(ClientMessage::SendSignal(CLIMSG_CHANNEL.into(), msg))?;

                if let BarMessage::Disconnecting = message {
                    self.stream.send(ClientMessage::Close)
                } else {
                    Ok(())
                }
            }
        }
    } else {
        pub struct BarMessager;

        impl BarMessager {
            pub fn new()-> io::Result<Self>  {
                Ok(Self)
            }

            pub fn send_message(&mut self, _: BarMessage) -> io::Result<()> {
                Ok(())
            }
        }
    }
}
