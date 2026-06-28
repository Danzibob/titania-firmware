#![no_std]
#![no_main]
#![warn(clippy::pedantic)]
#![allow(clippy::used_underscore_binding)]

mod baro;
mod buzz;
mod imu;
mod rickroll;

use defmt::{info, warn};
use defmt_rtt as _;

cfg_select! {
    debug_assertions => { use panic_probe as _; }
    _ => { use panic_reset as _; }
}

use static_cell::StaticCell;

// embassy
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_executor::Spawner;
use embassy_stm32::i2c::I2c;
use embassy_stm32::mode::Async;
use embassy_stm32::peripherals::{DMA1_CH2, DMA1_CH3, I2C1};
use embassy_stm32::time::Hertz;
use embassy_stm32::{bind_interrupts, dma, i2c};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::{Duration, Timer};

use crate::buzz::Buzzer;

pub type I2cBus = Mutex<NoopRawMutex, i2c::I2c<'static, Async, i2c::Master>>;
pub type I2cBusDevice = I2cDevice<'static, NoopRawMutex, i2c::I2c<'static, Async, i2c::Master>>;

static I2C_BUS: StaticCell<I2cBus> = StaticCell::new();

// We need to bind the interrupts for the I2C and DMA peripherals used by the I2C bus.
// The I2C1 Event and Error interrupts are used to wake the I2C bus task when an I2C transfer is complete or an error occurs.
// Likewise, the DMA1 Channel 2 and 3 interrupts are used to wake the I2C bus task when a DMA transfer is complete or an error occurs.
// This must match the DMA channels used by the I2C bus, which are configured in the I2c::new() call below.
bind_interrupts!(struct Irqs {
    I2C1 => i2c::EventInterruptHandler<I2C1>, i2c::ErrorInterruptHandler<I2C1>;
    DMA1_CHANNEL2_3 => dma::InterruptHandler<DMA1_CH2>, dma::InterruptHandler<DMA1_CH3>;
});

#[embassy_executor::main(
    executor = "embassy_stm32::executor::Executor",
    entry = "cortex_m_rt::entry"
)]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(embassy_stm32::Config::default());

    info!("Titania starting");

    // Initialize the buzzer.
    let mut buzzer = Buzzer::init(p.TIM3, p.PB0, p.PB1);

    // Initialize the I2C bus with I2C1, using PB6 and PB7 as the SCL and SDA pins.
    // And use DMA1 channel 2 and 3 for I2C1 TX and RX respectively.
    let i2c = I2c::new(
        p.I2C1,
        p.PB6,
        p.PB7,
        p.DMA1_CH2,
        p.DMA1_CH3,
        Irqs,
        embassy_stm32::i2c::Config::default(),
    );
    // Wrap the I2C bus in a mutex to allow sharing between multiple devices.
    let i2c_bus = I2C_BUS.init(Mutex::new(i2c));

    // Shared async I2C device handle for the BMI2xx.
    let i2c_imu = I2cDevice::new(i2c_bus);
    let i2c_baro = I2cDevice::new(i2c_bus);

    let mut bmi = imu::bmi(i2c_imu).await;
    let mut bmp = baro::bmp5(i2c_baro, embassy_stm32::gpio::Pull::Up)
        .await
        .expect("failed to initialise barometer");

    info!("beep!");
    buzzer.buzz(Duration::from_millis(100), Hertz(1000)).await;

    loop {
        Timer::after(Duration::from_millis(1000)).await;

        let data_imu = bmi.get_data().await.expect("Failed to read BMI2 data");
        let data_pres = bmp.meas_pres().await.expect("failed to read pressure");
        let data_temp = bmp.meas_temp().await.expect("failed to read temperature");

        defmt::println!(
            "Accel: x={} y={} z={}\nBaro: pressure={}hPa temp={}°C",
            data_imu.acc.x,
            data_imu.acc.y,
            data_imu.acc.z,
            data_pres,
            data_temp,
        );
    }
}
