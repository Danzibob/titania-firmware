use embassy_stm32::time::Hertz;
use embassy_stm32::timer::{Channel, GeneralInstance4Channel};
use embassy_stm32::timer::low_level::OutputPolarity;
use embassy_stm32::timer::simple_pwm::SimplePwm;
use embassy_time::{Duration, Timer};

/// A Buzzer struct that holds the peripherals needed to drive a piezo buzzer.
/// We should be able to specify the timer and pins we want to use for the buzzer, 
/// so let's make it generic over any TIMx that has 4 channels
/// and any pair of channels that we want to use for the buzzer.
pub struct Buzzer<'a, T: GeneralInstance4Channel> {
    pwm: SimplePwm<'a, T>,
    pos: Channel,
    neg: Channel,
}
impl<'a, T: GeneralInstance4Channel> Buzzer<'a, T> {
    pub fn new(pwm: SimplePwm<'a, T>, pos: Channel, neg: Channel) -> Self {
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
}
