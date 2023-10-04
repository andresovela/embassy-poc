use crate::bsp::{
    self, I2cDeviceOnSharedBus, IoExpanderIntGpio, IoExpanderResetGpio, PowerButtonGpio,
};
use aw9523b::Aw9523b;
use defmt::{info, Format};
use embassy_futures::select::*;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::channel::{Channel, Receiver, Sender};
use embassy_time::{Duration, Timer};
use static_cell::StaticCell;

type UiTaskChannel = Channel<NoopRawMutex, UiEvent, 2>;
type UiEventReceiver = Receiver<'static, NoopRawMutex, UiEvent, 2>;
type UiEventSender = Sender<'static, NoopRawMutex, UiEvent, 2>;

type UiBsp =
    bsp::ui::Ui<IoExpanderResetGpio, IoExpanderIntGpio, PowerButtonGpio, I2cDeviceOnSharedBus>;

static CHANNEL: StaticCell<UiTaskChannel> = StaticCell::new();

#[derive(Format)]
pub enum UiEvent {
    PowerOn,
    PowerOff,
}

pub struct UiTaskCtx<'a> {
    channel: &'a UiTaskChannel,
    i2c_device: I2cDeviceOnSharedBus,
    power_button_gpio: PowerButtonGpio,
    io_exp_reset_gpio: IoExpanderResetGpio,
    io_exp_int_gpio: IoExpanderIntGpio,
}

impl UiTaskCtx<'static> {
    /// Initializes and gets the communication channel for the UI task.
    ///
    /// # Panics
    ///
    /// This function panics if it gets called more than once.
    pub fn new(
        i2c_device: I2cDeviceOnSharedBus,
        power_button_gpio: PowerButtonGpio,
        io_exp_reset_gpio: IoExpanderResetGpio,
        io_exp_int_gpio: IoExpanderIntGpio,
    ) -> Self {
        Self {
            channel: CHANNEL.init(Channel::new()),
            i2c_device,
            power_button_gpio,
            io_exp_reset_gpio,
            io_exp_int_gpio,
        }
    }

    /// Gets a sender to the UI task event channel.
    pub fn sender(&self) -> UiEventSender {
        self.channel.sender()
    }
}

#[embassy_executor::task]
pub async fn ui_task(task_ctx: UiTaskCtx<'static>) {
    let io_expander = Aw9523b::new(task_ctx.i2c_device, bsp::i2c::AW9523B_I2C_ADDRESS);
    let mut ui = bsp::ui::Ui::new(
        io_expander,
        task_ctx.io_exp_reset_gpio,
        task_ctx.io_exp_int_gpio,
        task_ctx.power_button_gpio,
    );

    let event_receiver = task_ctx.channel.receiver();

    loop {
        let receive_event = event_receiver.recv();
        let timeout = Timer::after(Duration::from_millis(500));

        match select(receive_event, timeout).await {
            Either::First(event) => {
                info!("Got event {:?}", event);
            }
            Either::Second(_) => {
                if ui.is_initialized() {
                    on_idle(&mut ui).await;
                }
            }
        }
    }
}

async fn on_idle(ui: &mut UiBsp) {
    info!("Polling UI");

    if ui.is_power_pressed().unwrap() {
        info!("Power is pressed");
    }
}
