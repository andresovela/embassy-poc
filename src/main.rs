#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed};
use embassy_stm32::interrupt;
use embassy_stm32::i2c::{I2c};
use embassy_stm32::peripherals::{I2C2, DMA1_CH4, DMA1_CH5, PB2, PC5};
use embassy_stm32::time::Hertz;
use embassy_stm32::wdg::IndependentWatchdog;
use embassy_sync::mutex::Mutex;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_time::{Duration, Timer};
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use static_cell::StaticCell;
use board::ui::IoExpander;
use aw9523b::Aw9523b;

use {defmt_rtt as _, panic_probe as _};

mod tasks;
mod board;

type SharedI2cBus = I2c<'static, I2C2, DMA1_CH4, DMA1_CH5>;
type BoardIoExpander = IoExpander<Output<'static, PC5>, ExtiInput<'static, PB2>, I2cDevice<'static, ThreadModeRawMutex, SharedI2cBus>>;

static I2C2_BUS: StaticCell<Mutex::<ThreadModeRawMutex, SharedI2cBus>> = StaticCell::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let i2c2_irq = interrupt::take!(I2C2);

    let i2c2 = I2c::new(
        p.I2C2,
        p.PB10,
        p.PB11,
        i2c2_irq,
        p.DMA1_CH4,
        p.DMA1_CH5,
        Hertz(100_000),
        Default::default(),
    );

    let i2c2_bus = Mutex::<ThreadModeRawMutex, _>::new(i2c2);
    let i2c2_bus = I2C2_BUS.init(i2c2_bus);

    let ui_i2c_device = I2cDevice::new(i2c2_bus);
    let aw9523b = Aw9523b::new(ui_i2c_device, board::i2c::AW9523B_I2C_ADDRESS);

    let io_exp_int_gpio = Input::new(p.PB2, Pull::Up);
    let io_exp_int_gpio_exti = ExtiInput::new(io_exp_int_gpio, p.EXTI2);
    let io_exp_reset_gpio = Output::new(p.PC5, Level::Low, Speed::Low);
    let io_expander = IoExpander::new(aw9523b, io_exp_reset_gpio, io_exp_int_gpio_exti);

    let ui_task = tasks::ui_task::UiTask::single(io_expander);
    spawner
        .spawn(tasks::ui_task::ui_task(ui_task))
        .unwrap();

    let mut watchdog = IndependentWatchdog::new(p.IWDG, 10_000_000);
    unsafe {
        watchdog.unleash();
    }

    loop {
        Timer::after(Duration::from_millis(1000)).await;
        unsafe {
            watchdog.pet();
        }
    }
}
