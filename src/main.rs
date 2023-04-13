#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::wdg::IndependentWatchdog;
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

mod tasks;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let mut watchdog = IndependentWatchdog::new(p.IWDG, 10_000_000);
    unsafe { watchdog.unleash(); }

    spawner.spawn(tasks::led_task::led_task()).unwrap();
    spawner.spawn(tasks::button_task::button_task()).unwrap();

    loop {
        Timer::after(Duration::from_millis(1000)).await;
        unsafe { watchdog.pet(); }
    }
}
