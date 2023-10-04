use static_cell::StaticCell;

use embassy_futures::select::*;

use embassy_sync::{
    blocking_mutex::raw::{NoopRawMutex, RawMutex},
    channel::{Channel, Sender},
};

use embassy_time::{Duration, Timer};

pub type Inbox<M, MUTEX, const N: usize> = Sender<'static, MUTEX, M, N>;

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
    actor: A,
    channel: Channel<M, A::Message, QUEUE_SIZE>,
}

impl<A, M, const QUEUE_SIZE: usize, const IDLE_TIMEOUT_MS: u64>
    Actor<A, M, QUEUE_SIZE, IDLE_TIMEOUT_MS>
where
    A: ActorRuntime + 'static,
    M: RawMutex + 'static,
{
    pub const fn new(actor: A) -> Self {
        Self {
            actor,
            channel: Channel::new(),
        }
    }

    pub async fn run(&'static mut self) -> ! {
        self.actor.on_init().await;

        loop {
            let receive_message = self.channel.recv();
            let timeout = Timer::after(Duration::from_millis(IDLE_TIMEOUT_MS));

            match select(receive_message, timeout).await {
                Either::First(message) => self.actor.on_message_received(message).await,
                Either::Second(_) => self.actor.on_idle().await,
            }
        }
    }

    pub fn inbox(&'static self) -> Inbox<A::Message, M, QUEUE_SIZE> {
        self.channel.sender()
    }
}
