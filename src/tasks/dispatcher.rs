use super::{system, ui};
use actor::*;
use defmt::{info, Format};

pub const QUEUE_SIZE: usize = 10;
pub const IDLE_TIMEOUT_MS: u64 = 1000;

#[derive(Format)]
pub enum Message {
    System(system::Message),
    Ui(ui::Message),
}

pub struct Dispatcher {
    pub system_inbox: DynamicInbox<system::Message>,
    pub ui_inbox: DynamicInbox<ui::Message>,
}

impl Dispatcher {
    async fn on_system_message(&mut self, message: system::Message) {
        match message {
            system::Message::PowerOn => self.ui_inbox.send(ui::Message::PowerOn).await,
            system::Message::PowerOff => self.ui_inbox.send(ui::Message::PowerOff).await,
        }
    }

    async fn on_ui_message(&mut self, message: ui::Message) {
        info!("Power on");
    }
}

impl ActorRuntime for Dispatcher {
    type Message = Message;
    async fn on_init(&mut self) {}

    async fn on_idle(&mut self) {}

    async fn on_message_received(&mut self, message: Self::Message) {
        match message {
            Message::System(m) => self.on_system_message(m).await,
            Message::Ui(m) => self.on_ui_message(m).await,
        }
    }
}
