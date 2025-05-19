#![no_std]
#![no_main]

use panic_halt as _;
use rtt_target::rtt_init_log;

use cortex_m_rt::entry;
use critical_section_lock_mut::LockMut;
use embedded_hal::{
    delay::DelayNs,
    digital::{OutputPin, StatefulOutputPin},
};
use lsm303agr::{
    interface::I2cInterface, mode::MagOneShot, AccelMode, AccelOutputDataRate, Lsm303agr,
};
use microbit::{
    display::nonblocking::{BitImage, Display},
    hal::{
        delay::Delay,
        gpio::{p0::P0_00, Level, Output, PushPull},
        Timer,
    },
};
use micromath::F32Ext;

use microbit::{
    hal::twim,
    pac::{self, interrupt, twim0::frequency::FREQUENCY_A, TIMER1, TWIM0},
};

static DISPLAY: LockMut<Display<TIMER1>> = LockMut::new();
static SPEAKER: LockMut<Option<P0_00<Output<PushPull>>>> = LockMut::new();

#[entry]
fn main() -> ! {
    let still_image = BitImage::new(&[
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 1, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);

    let falling_image = BitImage::new(&[
        [0, 0, 1, 0, 0],
        [0, 0, 1, 0, 0],
        [0, 0, 1, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 1, 0, 0],
    ]);

    rtt_init_log!();

    let mut board = microbit::Board::take().unwrap();

    let mut timer = Timer::new(board.TIMER0);
    let mut timer2 = Timer::new(board.TIMER3);

    let display = Display::new(board.TIMER1, board.display_pins);
    DISPLAY.init(display);

    let speaker = board.speaker_pin.into_push_pull_output(Level::Low);
    SPEAKER.init(Some(speaker));

    unsafe {
        board.NVIC.set_priority(pac::Interrupt::TIMER1, 128);
        pac::NVIC::unmask(pac::Interrupt::TIMER1);
        pac::NVIC::unmask(pac::Interrupt::TIMER4);
    }

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

    let mut delay = Delay::new(board.SYST);
    let mut was_falling = true;
    loop {
        let mut is_falling = false;
        if let Some((x, y, z)) = get_data(&mut sensor) {
            let acc = calcuate_magnitude_of_acceleration(x, y, z);

            log::info!("acc <{}>", acc);

            if acc < 0.5 {
                is_falling = true;
            } else if acc > 1.0 {
                is_falling = false;
            }
        }

        if is_falling {
            SPEAKER.with_lock(|opt| {
                if let Some(speaker) = opt {
                    for _ in 0..200 {
                        speaker.set_high().unwrap();
                        delay.delay_us(500);
                        speaker.set_low().unwrap();
                        delay.delay_us(500);
                    }
                }
            });
        }
        DISPLAY.with_lock(|display| {
            log::info!("is falling <{}> was_falling <{}>", is_falling, was_falling);
            if is_falling && !was_falling {
                display.show(&falling_image);

                was_falling = true;
            } else if !is_falling && was_falling {
                display.show(&still_image);

                SPEAKER.with_lock(|opt| {
                    if let Some(speaker) = opt {
                        speaker.set_low().ok();
                    }
                });

                was_falling = false;
            }
        });
        timer2.delay_ms(50u32);
    }
}

type Sensor = Lsm303agr<I2cInterface<twim::Twim<TWIM0>>, MagOneShot>;

fn get_data(sensor: &mut Sensor) -> Option<(f32, f32, f32)> {
    if sensor.accel_status().unwrap().xyz_new_data() {
        let data = sensor.acceleration().unwrap();
        log::info!("x {} y {} z {}", data.x_mg(), data.y_mg(), data.z_mg());
        return Some((
            data.x_mg() as f32 / 1000.0,
            data.y_mg() as f32 / 1000.0,
            data.z_mg() as f32 / 1000.0,
        ));
    }

    None
}

fn calcuate_magnitude_of_acceleration(x: f32, y: f32, z: f32) -> f32 {
    (x * x + y * y + z * z).sqrt()
}

#[interrupt]
fn TIMER1() {
    DISPLAY.with_lock(|display| display.handle_display_event());
}

#[interrupt]
fn TIMER4() {
    let timer = unsafe { &*pac::TIMER4::ptr() };
    timer.events_compare[0].write(|w| unsafe { w.bits(0) });

    log::info!("Entering Timer 4 interrupt");
    SPEAKER.with_lock(|opt| {
        if let Some(speaker) = opt {
            let _ = speaker.toggle();
            let mut speaker_value = "Low";
            if speaker.is_set_high().unwrap() {
                speaker_value = "High";
            }
            log::info!("Speaker is <{}>", speaker_value)
        }
    });
}
