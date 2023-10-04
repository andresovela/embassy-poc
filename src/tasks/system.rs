use crate::bsp::PowerHoldGpio;
use defmt::{info, Format};
use embassy_futures::select::*;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::channel::{Channel, Receiver, Sender};
use embassy_time::{Duration, Timer};
use static_cell::StaticCell;

type SystemTaskChannel = Channel<NoopRawMutex, SystemEvent, 2>;
type SystemEventReceiver = Receiver<'static, NoopRawMutex, SystemEvent, 2>;
type SystemEventSender = Sender<'static, NoopRawMutex, SystemEvent, 2>;

static CHANNEL: StaticCell<SystemTaskChannel> = StaticCell::new();

#[derive(Format)]
pub enum SystemEvent {
    PowerOn,
    PowerOff,
}

pub struct SystemTaskCtx<'a> {
    channel: &'a SystemTaskChannel,
    power_hold_gpio: PowerHoldGpio,
}

impl SystemTaskCtx<'static> {
    /// Initializes and gets the communication channel for the system task.
    ///
    /// # Panics
    ///
    /// This function panics if it gets called more than once.
    pub fn new(power_hold_gpio: PowerHoldGpio) -> Self {
        Self {
            channel: CHANNEL.init(Channel::new()),
            power_hold_gpio,
        }
    }

    /// Gets a sender to the UI task event channel.
    pub fn sender(&self) -> SystemEventSender {
        self.channel.sender()
    }
}

#[embassy_executor::task]
pub async fn system_task(task_ctx: SystemTaskCtx<'static>) {
    let event_receiver = task_ctx.channel.receiver();

    loop {
        let receive_event = event_receiver.recv();
        let timeout = Timer::after(Duration::from_millis(1000));

        match select(receive_event, timeout).await {
            Either::First(event) => {
                info!("Got system event {:?}", event);
            }
            Either::Second(_) => {
                info!("System task idle");
            }
        }
    }
}
