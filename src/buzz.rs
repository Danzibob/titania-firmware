use embassy_stm32::time::Hertz;
use embassy_stm32::timer::low_level::OutputPolarity;
use embassy_stm32::timer::simple_pwm::SimplePwm;
use embassy_stm32::timer::{Channel, GeneralInstance4Channel};
use embassy_time::{Duration, Timer};

/// Piezo buzzer driven with Pulse Width Modulation.
///
/// Designed to be driven by two "opposing" PWM pins for extra volume.
/// Owns the peripherals needed to drive it.
///
/// `T` is generic over `TIM*` timers with 4 channels.
pub struct Buzzer<T: GeneralInstance4Channel> {
    // 'static means that this struct can live forever. this is okay
    // because the embassy_stm32::init gives us static implementations
    // (and you can get the controller back with destroy())
    pwm: SimplePwm<'static, T>,
    pos: Channel,
    neg: Channel,
}

impl<T: GeneralInstance4Channel> Buzzer<T> {
    /// Initialise a buzzer.
    ///
    /// - `pwm` is the configured PWM controller.
    /// - `pos` and `neg` are the channel indices in `pwm`'s timer which refer
    ///   to the active-high and active-low channels connected to the buzzer,
    ///   respectively.
    pub fn new(pwm: SimplePwm<'static, T>, pos: Channel, neg: Channel) -> Self {
        Self { pwm, pos, neg }
    }

    pub async fn buzz(&mut self, duration: Duration, frequency: Hertz) {
        self.pwm.set_frequency(frequency);
        {
            let mut pos = self.pwm.channel(self.pos);
            pos.set_polarity(OutputPolarity::ActiveHigh);
            pos.set_duty_cycle_percent(50);
            pos.enable();
        }
        {
            let mut neg = self.pwm.channel(self.neg);
            neg.set_polarity(OutputPolarity::ActiveLow);
            neg.set_duty_cycle_percent(50);
            neg.enable();
        }

        Timer::after(duration).await;

        {
            let mut pos = self.pwm.channel(self.pos);
            pos.disable();
        }
        {
            let mut neg = self.pwm.channel(self.neg);
            neg.disable();
        }
    }

    /// Deconstruct `self`, returning the underlying PWM handle for reuse.
    // channels are literally just constrained integers used for indexing,
    // so we don't need to bother with returning those
    pub fn destroy(self) -> SimplePwm<'static, T> {
        self.pwm
    }
}
