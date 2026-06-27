use crate::I2cBusDevice;

use bmi2::Bmi2;
use bmi2::interface::I2cInterface;
use bmi2::types::*;
use bmi2::{Builder, I2cAddr, config};

pub async fn bmi(i2c_dev: I2cBusDevice) -> Bmi2<I2cInterface<I2cBusDevice>, embassy_time::Delay> {
    // Initialize the BMI2xx sensor with the I2C device handle.
    let mut config_buf = [0u8; 512];
    Builder::i2c(
        i2c_dev,
        embassy_time::Delay,
        I2cAddr::Alternative,
        Burst::new(255),
    )
    .config(&config::BMI260_CONFIG_FILE)
    .pwr_ctrl(PwrCtrl {
        aux_en: false,
        gyr_en: true,
        acc_en: true,
        temp_en: false,
    })
    .acc_conf(AccConf {
        odr: Odr::Odr3p1,
        bwp: AccBwp::NormAvg4,
        filter_perf: PerfMode::Perf,
    })
    .acc_range(AccRange::Range16g)
    .gyr_conf(GyrConf {
        odr: Odr::Odr25,
        bwp: GyrBwp::Norm,
        noise_perf: PerfMode::Power,
        filter_perf: PerfMode::Power,
    })
    .gyr_range(GyrRange {
        range: GyrRangeVal::Range2000,
        ois_range: OisRange::Range2000,
    })
    .init(&mut config_buf)
    .await
    .expect("Failed to initialize BMI2")
}
