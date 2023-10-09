use crate::bsp::{I2cDeviceOnSharedBus, IoExpanderIntGpio, IoExpanderResetGpio, PowerButtonGpio};

use super::dispatcher;
use crate::bsp;
use actor::*;
use aw9523b::Aw9523b;
use buttons::{Buttons, Event, Id, Kind, Length, Ms, RepeatedPressMode};
use defmt::{info, Format};

pub const QUEUE_SIZE: usize = 3;
pub const IDLE_TIMEOUT_MS: u64 = 1000;

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
    buttons: Buttons<'static, Self>,
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

        let buttons_config = buttons::Config {
            short_press_duration: Ms(50),
            medium_press_duration: Ms(1000),
            long_press_duration: Ms(5000),
            very_long_press_duration: Ms(30000),
            hold_event_interval: Ms(100),
            repeated_press_threshold_duration: Ms(500),
            buttons_with_repeated_press_support: None,
            repeated_press_mode: RepeatedPressMode::Immediate,
            enable_raw_press_release_events: true,
        };

        let buttons: Buttons<'_, Self> = Buttons::new(buttons_config);

        Self {
            dispatcher_inbox,
            ui,
            buttons,
        }
    }

    fn on_power_off(&mut self) {
        info!("Power off");
    }

    fn on_power_on(&mut self) {
        info!("Power on");
    }
}

impl buttons::Handler for Ui {
    async fn on_event(&mut self, button: buttons::Id, event: Event) {
        info!("Got {} for button {}", event, button);
    }

    fn get_current_timestamp(&self) -> buttons::Ms {
        buttons::Ms(0)
    }
}

impl ActorRuntime for Ui {
    type Message = Message;
    async fn on_init(&mut self) {
        info!("UI init");
        self.ui.initialize().await.expect("Failed to initialize UI");
    }

    async fn on_idle(&mut self) {
        // let is_power_pressed = self.ui.is_power_pressed().unwrap();
        // let input = if is_power_pressed { Some(Id(0)) } else { None };
        // self.buttons.process_input(self, input);
    }

    async fn on_message_received(&mut self, message: Self::Message) {
        match message {
            Message::PowerOff => self.on_power_off(),
            Message::PowerOn => self.on_power_on(),
        }
    }
}
