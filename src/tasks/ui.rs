use crate::bsp::{I2cDeviceOnSharedBus, IoExpanderIntGpio, IoExpanderResetGpio, PowerButtonGpio};

use super::dispatcher;
use crate::bsp;
use actor::*;
use aw9523b::Aw9523b;
use defmt::{info, Format};

pub const QUEUE_SIZE: usize = 3;
pub const IDLE_TIMEOUT_MS: u64 = 30;

type UiBsp =
    bsp::ui::Ui<IoExpanderResetGpio, IoExpanderIntGpio, PowerButtonGpio, I2cDeviceOnSharedBus>;

#[derive(Format)]
pub enum Message {
    PowerOn,
    PowerOff,
}

pub struct Ui {
    dispatcher_inbox: DynamicInbox<dispatcher::Message>,
    ui: UiBsp,
}

impl Ui {
    pub fn new(
        dispatcher_inbox: DynamicInbox<dispatcher::Message>,
        i2c_device: I2cDeviceOnSharedBus,
        power_button_gpio: PowerButtonGpio,
        io_exp_reset_gpio: IoExpanderResetGpio,
        io_exp_int_gpio: IoExpanderIntGpio,
    ) -> Self {
        let io_expander = Aw9523b::new(i2c_device, bsp::i2c::AW9523B_I2C_ADDRESS);

        let ui = bsp::ui::Ui::new(
            io_expander,
            io_exp_reset_gpio,
            io_exp_int_gpio,
            power_button_gpio,
        );

        Self {
            dispatcher_inbox,
            ui,
        }
    }

    fn on_power_off(&mut self) {
        info!("Power off");
    }

    fn on_power_on(&mut self) {
        info!("Power on");
    }
}

impl ActorRuntime for Ui {
    type Message = Message;
    async fn on_init(&mut self) {}

    async fn on_idle(&mut self) {}

    async fn on_message_received(&mut self, message: Self::Message) {
        match message {
            Message::PowerOff => self.on_power_off(),
            Message::PowerOn => self.on_power_on(),
        }
    }
}
