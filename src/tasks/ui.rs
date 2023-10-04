use crate::bsp::{
    self, I2cDeviceOnSharedBus, IoExpanderIntGpio, IoExpanderResetGpio, PowerButtonGpio,
};

use actor::actor::*;
use defmt::{info, Format};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use static_cell::StaticCell;

type UiBsp =
    bsp::ui::Ui<IoExpanderResetGpio, IoExpanderIntGpio, PowerButtonGpio, I2cDeviceOnSharedBus>;

pub type UiActor = Actor<Ui, NoopRawMutex, 2, 1000>;
static ACTOR: StaticCell<UiActor> = StaticCell::new();

#[derive(Format)]
pub enum UiMessage {
    PowerOn,
    PowerOff,
}

pub struct Ui {
    pub i2c_device: I2cDeviceOnSharedBus,
    pub power_button_gpio: PowerButtonGpio,
    pub io_exp_reset_gpio: IoExpanderResetGpio,
    pub io_exp_int_gpio: IoExpanderIntGpio,
}

impl Ui {
    pub fn into_actor(self) -> &'static mut UiActor {
        let actor = Actor::new(self);
        ACTOR.init(actor)
    }
}

impl ActorRuntime for Ui {
    type Message = UiMessage;
    async fn on_init(&mut self) {}

    async fn on_idle(&mut self) {}

    async fn on_message_received(&mut self, message: Self::Message) {
        match message {
            UiMessage::PowerOff => info!("Power off"),
            UiMessage::PowerOn => info!("Power on"),
        }
    }
}
