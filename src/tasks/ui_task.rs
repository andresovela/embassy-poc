use defmt::info;
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::{Channel, Receiver, Sender};
use embassy_time::{Duration, Timer};
use static_cell::StaticCell;
use crate::{SharedI2cBus, BoardIoExpander};
use crate::board::ui::IoExpander;

type UiTaskChannel = Channel<NoopRawMutex, UiTaskEvent, 2>;
type UiEventReceiver = Receiver<'static, NoopRawMutex, UiTaskEvent, 2>;
type UiEventSender = Sender<'static, NoopRawMutex, UiTaskEvent, 2>;

type I2cDriver = I2cDevice<'static, ThreadModeRawMutex, SharedI2cBus>;

static CHANNEL: StaticCell<UiTaskChannel> = StaticCell::new();

pub enum UiTaskEvent {
    PowerOn,
    PowerOff,
}

pub struct UiTask<'a> {
    channel: &'a UiTaskChannel,
    io_expander: BoardIoExpander,
}

impl UiTask<'static> {
    /// Initializes and gets the communication channel for the UI task.
    ///
    /// # Panics
    ///
    /// This function panics if it gets called more than once.
    pub fn single(io_expander: BoardIoExpander) -> Self {
        Self {
            channel: CHANNEL.init(Channel::new()),
            io_expander,
        }
    }

    /// Gets a sender to the UI task event channel.
    pub fn sender(&self) -> UiEventSender {
        self.channel.sender()
    }

    async fn init(&mut self) {
    }

    async fn idle(&self) {}

    async fn process_event(&self, event: UiTaskEvent) {
        use UiTaskEvent::*;
        match event {
            PowerOn => 0,
            PowerOff => 1,
        };
    }
}

#[embassy_executor::task]
pub async fn ui_task(task: UiTask<'static>) {
    task.init().await;

    let event_receiver = task.channel.receiver();

    loop {
        let event = event_receiver.recv().await;
        task.process_event(event).await;

        info!("Button poll");
        Timer::after(Duration::from_millis(500)).await;
    }
}
