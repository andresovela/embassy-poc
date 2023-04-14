use defmt::info;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::channel::{Channel, Receiver, Sender};
use embassy_time::{Duration, Timer};
use static_cell::StaticCell;

type UiTaskChannel = Channel<NoopRawMutex, UiTaskEvent, 2>;
type UiEventReceiver = Receiver<'static, NoopRawMutex, UiTaskEvent, 2>;
type UiEventSender = Sender<'static, NoopRawMutex, UiTaskEvent, 2>;

static CHANNEL: StaticCell<UiTaskChannel> = StaticCell::new();

pub enum UiTaskEvent {
    PowerOn,
    PowerOff,
}

pub struct UiTask<'a> {
    channel: &'a UiTaskChannel,
}

impl UiTask<'static> {
    /// Initializes and gets the communication channel for the UI task.
    ///
    /// # Panics
    ///
    /// This function panics if it gets called more than once.
    pub fn single() -> Self {
        Self { channel: CHANNEL.init(Channel::new()) }
    }

    /// Gets a sender to the UI task event channel.
    pub fn sender(&self) -> UiEventSender {
        self.channel.sender()
    }

    async fn init(&self) {}

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
