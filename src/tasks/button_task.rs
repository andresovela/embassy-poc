use defmt::info;
use embassy_time::{Duration, Timer};

#[embassy_executor::task]
pub async fn button_task() {
    loop {
        info!("Button poll");
        Timer::after(Duration::from_millis(500)).await;
    }
}
