use embassy_stm32::Peri;
use embassy_stm32::gpio::OutputType;
use embassy_stm32::time::Hertz;
use embassy_stm32::timer::low_level::{CountingMode, OutputPolarity};
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm, SimplePwmChannel};
use embassy_stm32::timer::{Ch3, Ch4, TimerChannel, TimerPin};
use embassy_time::{Duration, Timer as EmbTimer};

pub type Timer = embassy_stm32::peripherals::TIM3;
pub type PosChan = Ch3;
pub type NegChan = Ch4;

/// Piezo buzzer driven with Pulse Width Modulation.
///
/// Designed to be driven by two "opposing" PWM pins for extra volume.
/// Owns the peripherals needed to drive it.
///
/// This implementation uses [`Timer`] with [`PosChan`] and [`NegChan`].
pub struct Buzzer {
    // 'static means that this struct can live forever. this is okay
    // because the embassy_stm32::init gives us static implementations
    // (and you can get the controller back with destroy())
    pwm: SimplePwm<'static, Timer>,
}

impl Buzzer {
    /// Initialise the buzzer.
    ///
    /// `timer` is the (owned) [`Timer`] peripheral handle.
    /// `*_pin` are the (owned) pins for channels [`PosChan`] and [`NegChan`].
    pub fn init(
        timer: Peri<'static, Timer>,
        pos_pin: Peri<'static, impl TimerPin<Timer, PosChan>>,
        neg_pin: Peri<'static, impl TimerPin<Timer, NegChan>>,
    ) -> Self {
        let pos_pin = PwmPin::new(pos_pin, OutputType::PushPull);
        let neg_pin = PwmPin::new(neg_pin, OutputType::PushPull);
        let mut pwm = SimplePwm::new(
            timer,
            // note: because of how embassy have made this constructor work, the ordering
            // of these has to change if you change what channels of the timer you're using
            // (also stopping this struct from being truly generic :( )
            // it should be a builder with a generic param really
            None,
            None,
            Some(pos_pin),
            Some(neg_pin),
            Hertz(1000),
            CountingMode::EdgeAlignedUp,
        );

        // configure as positive and negative
        {
            let mut pos = pwm.channel(PosChan::CHANNEL);
            pos.set_polarity(OutputPolarity::ActiveHigh);
        }
        {
            let mut neg = pwm.channel(NegChan::CHANNEL);
            neg.set_polarity(OutputPolarity::ActiveLow);
        }

        Self { pwm }
    }

    fn for_both_channels(&mut self, mut func: impl FnMut(SimplePwmChannel<'_, Timer>)) {
        func(self.pwm.channel(PosChan::CHANNEL));
        func(self.pwm.channel(NegChan::CHANNEL));
    }

    /// Buzz for `duration` with the given `frequency`/pitch.
    pub async fn buzz(&mut self, duration: Duration, frequency: Hertz) {
        self.pwm.set_frequency(frequency);

        self.for_both_channels(|mut chan| {
            chan.set_duty_cycle_percent(50);
            chan.enable();
        });

        EmbTimer::after(duration).await;

        self.for_both_channels(|mut chan| chan.disable());
    }

    /// Deconstruct `self`, returning the underlying PWM handle for reuse.
    #[allow(dead_code)]
    pub fn destroy(self) -> SimplePwm<'static, Timer> {
        self.pwm
    }
}
