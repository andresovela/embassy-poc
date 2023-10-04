use defmt::{info, Format};
use embassy_futures::select::*;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::channel::{Channel, Receiver, Sender};
use embassy_time::{Duration, Timer};
use static_cell::StaticCell;

use super::*;

type DispatcherTaskChannel = Channel<NoopRawMutex, DispatcherEvent, 2>;
type DispatcherEventReceiver = Receiver<'static, NoopRawMutex, DispatcherEvent, 2>;
type DispatcherEventSender = Sender<'static, NoopRawMutex, DispatcherEvent, 2>;

static CHANNEL: StaticCell<DispatcherTaskChannel> = StaticCell::new();

#[derive(Format)]
pub enum DispatcherEvent {
    Ui(ui::UiEvent),
    System(system::SystemEvent),
}

pub struct DispatcherTaskCtx<'a> {
    channel: &'a DispatcherTaskChannel,
}

impl DispatcherTaskCtx<'static> {
    /// Initializes and gets the communication channel for the dispatcher task.
    ///
    /// # Panics
    ///
    /// This function panics if it gets called more than once.
    pub fn new() -> Self {
        Self {
            channel: CHANNEL.init(Channel::new()),
        }
    }

    /// Gets a sender to the dispatcher task event channel.
    pub fn sender(&self) -> DispatcherEventSender {
        self.channel.sender()
    }
}

#[embassy_executor::task]
pub async fn dispatcher_task(task_ctx: DispatcherTaskCtx<'static>) {
    let event_receiver = task_ctx.channel.receiver();

    loop {
        let receive_event = event_receiver.recv();
        let timeout = Timer::after(Duration::from_millis(1000));

        match select(receive_event, timeout).await {
            Either::First(event) => {
                info!("Got dispatcher event {:?}", event);
            }
            Either::Second(_) => {
                // Nothing to do
            }
        }
    }
}

pub async fn dispatch(event: DispatcherEvent) {
    match event {
        DispatcherEvent::Ui(e) => {}
        DispatcherEvent::System(e) => {}
    }
}
