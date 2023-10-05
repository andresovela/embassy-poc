use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed};
use embassy_stm32::i2c::I2c;
use embassy_stm32::interrupt;
use embassy_stm32::peripherals::{DMA1_CH4, DMA1_CH5, I2C2, IWDG, PA1, PA2, PB2, PC5};
use embassy_stm32::time::Hertz;
use embassy_stm32::wdg::IndependentWatchdog;
use embassy_stm32::Peripherals;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::mutex::Mutex;
use static_cell::StaticCell;

pub mod power;
pub mod ui;

pub mod i2c {
    pub const AW9523B_I2C_ADDRESS: u8 = 0x5B;
}

pub type SharedI2cBus = I2c<'static, I2C2, DMA1_CH4, DMA1_CH5>;
pub type IoExpanderResetGpio = Output<'static, PC5>;
pub type IoExpanderIntGpio = ExtiInput<'static, PB2>;
pub type PowerButtonGpio = Input<'static, PA2>;
pub type PowerHoldGpio = Output<'static, PA1>;
pub type I2cDeviceOnSharedBus = I2cDevice<'static, ThreadModeRawMutex, SharedI2cBus>;
pub type Watchdog = IndependentWatchdog<'static, IWDG>;

static I2C2_BUS: StaticCell<Mutex<ThreadModeRawMutex, SharedI2cBus>> = StaticCell::new();

pub struct EcospeakerV1<'a> {
    pub shared_i2c_bus: &'a mut Mutex<ThreadModeRawMutex, SharedI2cBus>,
    pub power_button_gpio: PowerButtonGpio,
    pub power_hold_gpio: PowerHoldGpio,
    pub io_exp_reset_gpio: IoExpanderResetGpio,
    pub io_exp_int_gpio: IoExpanderIntGpio,
    pub watchdog: Watchdog,
}

impl EcospeakerV1<'static> {
    pub fn new(p: Peripherals) -> Self {
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
        let shared_i2c_bus: &mut Mutex<ThreadModeRawMutex, I2c<I2C2, DMA1_CH4, DMA1_CH5>> =
            I2C2_BUS.init(i2c2_bus);

        let power_button_gpio = Input::new(p.PA2, Pull::Up);
        let power_hold_gpio = Output::new(p.PA1, Level::Low, Speed::Low);
        let io_exp_int_gpio = Input::new(p.PB2, Pull::Up);
        let io_exp_int_gpio = ExtiInput::new(io_exp_int_gpio, p.EXTI2);
        let io_exp_reset_gpio = Output::new(p.PC5, Level::Low, Speed::Low);

        let watchdog = IndependentWatchdog::new(p.IWDG, 10_000_000);

        Self {
            shared_i2c_bus,
            power_button_gpio,
            power_hold_gpio,
            io_exp_reset_gpio,
            io_exp_int_gpio,
            watchdog,
        }
    }
}
