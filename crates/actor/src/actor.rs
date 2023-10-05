use static_cell::StaticCell;

use embassy_futures::select::*;

use embassy_sync::{
    blocking_mutex::raw::{NoopRawMutex, RawMutex},
    channel::{Channel, DynamicSender, Sender},
};

use embassy_time::{Duration, Timer};

pub type Inbox<M, MUTEX, const N: usize> = Sender<'static, MUTEX, M, N>;
pub type DynamicInbox<M> = DynamicSender<'static, M>;

pub trait ActorRuntime {
    type Message;

    async fn on_init(&mut self);

    async fn on_idle(&mut self);

    async fn on_message_received(&mut self, message: Self::Message);
}

pub struct Actor<A, M, const QUEUE_SIZE: usize, const IDLE_TIMEOUT_MS: u64>
where
    A: ActorRuntime + 'static,
    M: RawMutex + 'static,
{
    actor: StaticCell<A>,
    channel: Channel<M, A::Message, QUEUE_SIZE>,
}

impl<A, M, const QUEUE_SIZE: usize, const IDLE_TIMEOUT_MS: u64>
    Actor<A, M, QUEUE_SIZE, IDLE_TIMEOUT_MS>
where
    A: ActorRuntime + 'static,
    M: RawMutex + 'static,
{
    pub const fn new() -> Self {
        Self {
            actor: StaticCell::new(),
            channel: Channel::new(),
        }
    }

    pub async fn run(&'static self, actor: A) -> ! {
        let actor = self.actor.init(actor);
        actor.on_init().await;

        loop {
            let receive_message = self.channel.recv();
            let timeout = Timer::after(Duration::from_millis(IDLE_TIMEOUT_MS));

            match select(receive_message, timeout).await {
                Either::First(message) => actor.on_message_received(message).await,
                Either::Second(_) => actor.on_idle().await,
            }
        }
    }

    pub fn inbox(&'static self) -> Inbox<A::Message, M, QUEUE_SIZE> {
        self.channel.sender()
    }

    pub fn dyn_inbox(&'static self) -> DynamicInbox<A::Message> {
        self.channel.sender().into()
    }
}
