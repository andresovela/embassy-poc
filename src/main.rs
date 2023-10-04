#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(async_fn_in_trait)]

use actor::*;
use defmt::*;
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};

use {defmt_rtt as _, panic_probe as _};

mod bsp;
mod tasks;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p: embassy_stm32::Peripherals = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let mut board = bsp::EcospeakerV1::new(p);

    let io_expander_i2c = I2cDevice::new(board.shared_i2c_bus);
    let ui_task = tasks::ui::UiTaskCtx::new(
        io_expander_i2c,
        board.power_button_gpio,
        board.io_exp_reset_gpio,
        board.io_exp_int_gpio,
    );
    spawner.spawn(tasks::ui::ui_task(ui_task)).unwrap();

    unsafe {
        board.watchdog.unleash();
    }

    loop {
        Timer::after(Duration::from_millis(1000)).await;
        unsafe {
            board.watchdog.pet();
        }
    }
}
