#![no_std]
#![no_main]

mod buzz;
mod rickroll;

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::gpio::OutputType;
use embassy_stm32::time::Hertz;
use embassy_stm32::timer::low_level::CountingMode;
use embassy_stm32::timer::simple_pwm::{SimplePwm,PwmPin};
use embassy_stm32::timer::Channel;
use embassy_time::{Duration, Timer};

cfg_select! {
    // if debug assertions are turned on, panic using the connected probe
    debug_assertions => { use panic_probe as _; }
    // otherwise, just reset the chip
    _ => { use panic_reset as _; }
}

const BUZZ_DURATION: Duration = Duration::from_millis(500);
const BUZZ_PAUSE:    Duration = Duration::from_millis(500);

#[embassy_executor::main(executor = "embassy_stm32::executor::Executor", entry = "cortex_m_rt::entry")]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    info!("Titania starting");

    // Initialize the buzzer with TIM3 ch3&4, using PB0 and PB1 as the output pins.
    let mut buzzer = buzz::Buzzer::new(
        SimplePwm::new(
            p.TIM3, 
            None,
            None,
            Some(PwmPin::new(p.PB0, OutputType::PushPull)),
            Some(PwmPin::new(p.PB1, OutputType::PushPull)),
            Hertz(1000),
            CountingMode::EdgeAlignedUp
        ),
        Channel::Ch3,
        Channel::Ch4
    );

    loop {
        buzzer.buzz(BUZZ_DURATION, Hertz(1_000)).await;
        Timer::after(Duration::from_millis(2000)).await;

        for note in rickroll::RICKROLL.iter() {
            if note.pitch != rickroll::Pitch::REST {
                buzzer.buzz(
                    Duration::from_millis((rickroll::BASIS * note.duration as f32) as u64),
                    Hertz(note.pitch as u32)
                ).await;
            } else {
                Timer::after(Duration::from_millis((rickroll::BASIS * note.duration as f32) as u64)).await;
            }
        }
    }
}
