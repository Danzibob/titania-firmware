#![no_std]
#![no_main]

mod buzz;
mod i2c_bus;
mod imu;
mod rickroll;

use defmt::*;
use defmt_rtt as _;

cfg_select! {
    debug_assertions => { use panic_probe as _; }
    _ => { use panic_reset as _; }
}

use static_cell::StaticCell;

// embassy
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_executor::Spawner;
use embassy_stm32::gpio::OutputType;
use embassy_stm32::i2c::I2c;
use embassy_stm32::peripherals::*;
use embassy_stm32::time::Hertz;
use embassy_stm32::timer::Channel;
use embassy_stm32::timer::low_level::CountingMode;
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::{bind_interrupts, dma, i2c};
use embassy_sync::mutex::Mutex;
use embassy_time::{Duration, Timer};

static I2C_BUS: StaticCell<i2c_bus::I2cBus> = StaticCell::new();

bind_interrupts!(struct Irqs {
    I2C1 => i2c::EventInterruptHandler<I2C1>, i2c::ErrorInterruptHandler<I2C1>;
    DMA1_CHANNEL2_3 => dma::InterruptHandler<DMA1_CH2>, dma::InterruptHandler<DMA1_CH3>;
});

#[embassy_executor::main(
    executor = "embassy_stm32::executor::Executor",
    entry = "cortex_m_rt::entry"
)]
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
            CountingMode::EdgeAlignedUp,
        ),
        Channel::Ch3,
        Channel::Ch4,
    );

    // Initialize the I2C bus with I2C1, using PB6 and PB7 as the SCL and SDA pins.
    let i2c = I2c::new(
        p.I2C1,
        p.PB6,
        p.PB7,
        p.DMA1_CH2,
        p.DMA1_CH3,
        Irqs,
        Default::default(),
    );
    // Wrap the I2C bus in a mutex to allow sharing between multiple devices.
    let i2c_bus = I2C_BUS.init(Mutex::new(i2c));

    // Shared async I2C device handle for the BMI2xx.
    let i2c_dev1 = I2cDevice::new(i2c_bus);
    let _i2c_dev2 = I2cDevice::new(i2c_bus);

    let mut bmi = imu::bmi(i2c_dev1).await;

    buzzer.buzz(Duration::from_millis(100), Hertz(1000)).await;

    loop {
        Timer::after(Duration::from_millis(1000)).await;

        let data = bmi.get_data().await.expect("Failed to read BMI2 data");

        info!("Accel: x={} y={} z={}", data.acc.x, data.acc.y, data.acc.z);
    }
}
