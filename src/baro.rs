use bmp5xx::{Bmp5xx, int::Interrupt};
use defmt::debug;
use embassy_stm32::gpio::Pull as GpioPull;

use crate::I2cBusDevice;

/// Initialise the onboard barometer (a BMP580).
///
/// Calls [`Bmp5xx::init`] for you and enables latched push-pull interrupts with the
/// provided `polarity`.
///
/// # Errors
///
/// Returns a device error if initialisation failed or configuring interrupts failed.
pub async fn bmp5(
    i2c: I2cBusDevice,
    polarity: GpioPull,
) -> Result<Bmp5xx<I2cBusDevice, embassy_time::Delay>, bmp5xx::error::Error> {
    let mut baro = Bmp5xx::new(i2c, embassy_time::Delay, 0x46);
    baro.init().await?;

    debug!(
        "enabling latched push/pull interrupts with Pull-{:?}",
        polarity
    );
    let interrupts = Interrupt::default()
        .enable(true)
        .pin(bmp5xx::int::IntPin::PushPull)
        .mode(bmp5xx::int::IntMode::Latched)
        .polarity(match polarity {
            GpioPull::None => {
                panic!("BMP5xx barometers don't support no push or pull behaviour for interrupts")
            }
            GpioPull::Up => bmp5xx::int::IntPolarity::ActiveHigh,
            GpioPull::Down => bmp5xx::int::IntPolarity::ActiveLow,
        });
    baro.int(interrupts).await?;

    Ok(baro)
}
