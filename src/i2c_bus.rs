use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_stm32::i2c;
use embassy_stm32::mode::Async;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;

pub type I2cBus = Mutex<NoopRawMutex, i2c::I2c<'static, Async, i2c::Master>>;
pub type BusDevice = I2cDevice<'static, NoopRawMutex, i2c::I2c<'static, Async, i2c::Master>>;
