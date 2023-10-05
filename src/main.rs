#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(async_fn_in_trait)]

use actor::*;
use defmt::*;
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_executor::Spawner;
use embassy_futures::join::join3;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;

use {defmt_rtt as _, panic_probe as _};

use tasks::{dispatcher, system, ui};

mod bsp;
mod tasks;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p: embassy_stm32::Peripherals = embassy_stm32::init(Default::default());
    info!("Hello World!");

    static DISPATCHER: Actor<
        dispatcher::Dispatcher,
        ThreadModeRawMutex,
        { dispatcher::QUEUE_SIZE },
        { dispatcher::IDLE_TIMEOUT_MS },
    > = Actor::new();

    static SYSTEM: Actor<
        system::System,
        ThreadModeRawMutex,
        { system::QUEUE_SIZE },
        { system::IDLE_TIMEOUT_MS },
    > = Actor::new();

    static UI: Actor<ui::Ui, ThreadModeRawMutex, { ui::QUEUE_SIZE }, { ui::IDLE_TIMEOUT_MS }> =
        Actor::new();

    let board = bsp::EcospeakerV1::new(p);

    let io_expander_i2c = I2cDevice::new(board.shared_i2c_bus);

    let dispatcher = tasks::dispatcher::Dispatcher {
        system_inbox: SYSTEM.dyn_inbox(),
        ui_inbox: UI.dyn_inbox(),
    };

    let system = tasks::system::System::new(
        DISPATCHER.dyn_inbox(),
        board.power_hold_gpio,
        board.watchdog,
    );

    let ui = tasks::ui::Ui::new(
        DISPATCHER.dyn_inbox(),
        io_expander_i2c,
        board.power_button_gpio,
        board.io_exp_reset_gpio,
        board.io_exp_int_gpio,
    );

    let dispatcher = DISPATCHER.run(dispatcher);
    let system = SYSTEM.run(system);
    let ui = UI.run(ui);

    join3(dispatcher, system, ui).await;
}
