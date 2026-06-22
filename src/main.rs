#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main(executor = "embassy_stm32::executor::Executor", entry = "cortex_m_rt::entry")]
async fn main(_spawner: Spawner) {
    info!("Hello World!");
}