#![no_std]
#![no_main]

mod buzz;

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};

#[cfg(feature = "probe")]
use panic_probe as _;

#[cfg(feature = "release-build")]
use panic_reset as _;

const BUZZ_DURATION: Duration = Duration::from_millis(500);
const BUZZ_PAUSE:    Duration = Duration::from_millis(500);

#[embassy_executor::main(executor = "embassy_stm32::executor::Executor", entry = "cortex_m_rt::entry")]
async fn main(_spawner: Spawner) {
    let mut p = embassy_stm32::init(Default::default());
    info!("Titania starting");

    loop {
        buzz::buzz(p.TIM3.reborrow(), p.PB0.reborrow(), p.PB1.reborrow(), BUZZ_DURATION).await;
        Timer::after(BUZZ_PAUSE).await;
    }
}
