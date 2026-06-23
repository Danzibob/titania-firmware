use embassy_stm32::gpio::OutputType;
use embassy_stm32::peripherals::{PB0, PB1, TIM3};
use embassy_stm32::time::Hertz;
use embassy_stm32::timer::low_level::{CountingMode, OutputPolarity};
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::Peri;
use embassy_time::{Duration, Timer};

/// Drive the piezo buzzer between PB0 (TIM3_CH3) and PB1 (TIM3_CH4) at 1 kHz
/// for `duration`. CH3 and CH4 are driven with inverted polarity so the full
/// supply voltage appears across the piezo on each half-cycle.
pub async fn buzz(tim3: Peri<'_, TIM3>, pb0: Peri<'_, PB0>, pb1: Peri<'_, PB1>, duration: Duration) {
    let ch3_pin = PwmPin::new(pb0, OutputType::PushPull);
    let ch4_pin = PwmPin::new(pb1, OutputType::PushPull);

    let mut pwm = SimplePwm::new(
        tim3,
        None,
        None,
        Some(ch3_pin),
        Some(ch4_pin),
        Hertz(2_000),
        CountingMode::EdgeAlignedUp,
    );

    {
        let mut ch = pwm.ch3();
        ch.set_duty_cycle_percent(50);
        ch.enable();
    }
    {
        let mut ch = pwm.ch4();
        ch.set_polarity(OutputPolarity::ActiveLow);
        ch.set_duty_cycle_percent(50);
        ch.enable();
    }

    Timer::after(duration).await;
}
