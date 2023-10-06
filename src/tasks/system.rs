use crate::bsp::{power::Power, PowerHoldGpio, Watchdog};

use super::dispatcher;
use actor::*;
use defmt::{info, Format};

pub const QUEUE_SIZE: usize = 3;
pub const IDLE_TIMEOUT_MS: u64 = 1000;

#[derive(Format)]
pub enum Message {
    PowerOn,
    PowerOff,
}

pub struct System {
    dispatcher_inbox: DynamicInbox<dispatcher::Message>,
    power: Power,
    watchdog: Watchdog,
}

impl System {
    pub fn new(
        dispatcher_inbox: DynamicInbox<dispatcher::Message>,
        power_hold_gpio: PowerHoldGpio,
        watchdog: Watchdog,
    ) -> Self {
        let power = Power::new(power_hold_gpio);
        Self {
            dispatcher_inbox,
            power,
            watchdog,
        }
    }

    fn on_power_off(&mut self) {
        info!("Power off");
        self.power.release();
    }

    fn on_power_on(&mut self) {
        info!("Power on");
        self.power.hold();
    }
}

impl ActorRuntime for System {
    type Message = Message;
    async fn on_init(&mut self) {
        info!("System init");
        unsafe {
            self.watchdog.unleash();
        }
    }

    async fn on_idle(&mut self) {
        unsafe {
            self.watchdog.pet();
        }
    }

    async fn on_message_received(&mut self, message: Self::Message) {
        match message {
            Message::PowerOff => self.on_power_off(),
            Message::PowerOn => self.on_power_on(),
        }
    }
}
