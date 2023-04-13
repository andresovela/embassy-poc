use defmt::info;
use embassy_time::{Duration, Timer};

#[embassy_executor::task]
pub async fn led_task() {
    loop {
        info!("LED on");
        Timer::after(Duration::from_millis(1000)).await;

        info!("LED off");
        Timer::after(Duration::from_millis(1000)).await;
    }
}