#![no_std]
#![no_main]

use rtt_target::rtt_init_log;
use panic_halt as _;

use cortex_m_rt::entry;
use embedded_hal::delay::DelayNs;
use lsm303agr::{
    interface::I2cInterface, mode::MagOneShot, AccelMode, AccelOutputDataRate, Lsm303agr,
};
use microbit::hal::{timer, Timer};
use critical_section_lock_mut::LockMut;

use microbit::{
    hal::twim,
    pac::{twim0::frequency::FREQUENCY_A, TWIM0},
};

#[entry]
fn main() -> ! {
    rtt_init_log!();

    let board = microbit::Board::take().unwrap();
    let mut timer = Timer::new(board.TIMER0);

    // source: https://github.com/nrf-rs/microbit/blob/main/examples/magnetometer/src/main.rs
    let i2c = { twim::Twim::new(board.TWIM0, board.i2c_internal.into(), FREQUENCY_A::K100) };

    let mut sensor = Lsm303agr::new_with_i2c(i2c);

    if let Ok(id) = sensor.accelerometer_id() {
        if !id.is_correct() {
            log::error!("Accelerometer had Unexpected ID {:#x}", id.raw());
        }
    } else {
        log::error!("Error getting accelerometer ID");
    }
    sensor.init().unwrap();

    log::info!("normal mode");
    sensor
        .set_accel_mode_and_odr(&mut timer, AccelMode::Normal, AccelOutputDataRate::Hz50)
        .unwrap();
    timer.delay_ms(1000_u32);
    get_data(&mut sensor);

    loop {
        timer.delay_ms(100_u32);
        get_data(&mut sensor);
    }
}

type Sensor = Lsm303agr<I2cInterface<twim::Twim<TWIM0>>, MagOneShot>;

fn get_data(sensor: &mut Sensor) {
    loop {
        if sensor.accel_status().unwrap().xyz_new_data() {
            let data = sensor.acceleration().unwrap();
            log::info!("x {} y {} z {}", data.x_mg(), data.y_mg(), data.z_mg());
            return;
        }
    }
}
