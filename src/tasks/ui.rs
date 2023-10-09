use crate::bsp::{
    self, I2cDeviceOnSharedBus, IoExpanderIntGpio, IoExpanderResetGpio, PowerButtonGpio,
};
use aw9523b::Aw9523b;
use buttons::{Buttons, Event, Id, Ms, RepeatedPressMode};
use defmt::info;
use embassy_time::{Duration, Timer};

type UiBsp =
    bsp::ui::Ui<IoExpanderResetGpio, IoExpanderIntGpio, PowerButtonGpio, I2cDeviceOnSharedBus>;

pub struct Ui {
    hw: UiBsp,
}

impl Ui {
    pub fn new(
        i2c_device: I2cDeviceOnSharedBus,
        power_button_gpio: PowerButtonGpio,
        io_exp_reset_gpio: IoExpanderResetGpio,
        io_exp_int_gpio: IoExpanderIntGpio,
    ) -> Self {
        let io_expander = Aw9523b::new(i2c_device, bsp::i2c::address::AW9523B);

        let hw = bsp::ui::Ui::new(
            io_expander,
            io_exp_reset_gpio,
            io_exp_int_gpio,
            power_button_gpio,
        );

        Self { hw }
    }
}

#[embassy_executor::task]
pub async fn task(mut ui: Ui) {
    let mut buttons: Buttons<'_, Ui> = Buttons::new(buttons::Config {
        short_press_duration: Ms(50),
        medium_press_duration: Ms(1000),
        long_press_duration: Ms(5000),
        very_long_press_duration: Ms(30000),
        hold_event_interval: Ms(100),
        repeated_press_threshold_duration: Ms(500),
        buttons_with_repeated_press_support: None,
        repeated_press_mode: RepeatedPressMode::Immediate,
        enable_raw_press_release_events: true,
    });

    info!("Initializing hardware");
    ui.hw.initialize().await.unwrap();

    loop {
        info!("tick");
        let input = if ui.hw.is_bt_pressed().await.unwrap() {
            Some(Id(0))
        } else {
            None
        };

        buttons.process_input(&mut ui, input).await;
        Timer::after(Duration::from_millis(30)).await;
    }
}

impl buttons::Handler for Ui {
    async fn on_event(&mut self, button: buttons::Id, event: Event) {
        info!("Got {} for button {}", event, button);
    }

    fn get_current_timestamp(&self) -> Ms {
        Ms(embassy_time::Instant::now().as_millis())
    }
}
