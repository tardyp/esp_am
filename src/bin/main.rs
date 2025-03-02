#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::Instant;
use esp_backtrace as _;
use esp_hal::mcpwm::operator::PwmPinConfig;
use esp_hal::mcpwm::timer::PwmWorkingMode;
use esp_hal::mcpwm::{McPwm, PeripheralClockConfig};
use esp_hal::time::Rate;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::clock::CpuClock;
use log::info;
extern crate alloc;

// Lookup table for sine values (scaled to fit in u16)
const SINE_TABLE: [u16; 256] = 
[32767, 33571, 34374, 35177, 35978, 36778, 37574, 38368, 39159, 39946, 40728, 41506, 42278, 43045, 43805, 44559, 45306, 46045, 46776, 47499, 48213, 48917, 49612, 50297, 50971, 51634, 52286, 52926, 53554, 54169, 54771, 55361, 55936, 56498, 57045, 57578, 58096, 58598, 59085, 59556, 60011, 60450, 60872, 61277, 61664, 62035, 62388, 62722, 63039, 63338, 63618, 63880, 64123, 64347, 64552, 64737, 64904, 65051, 65179, 65287, 65376, 65445, 65494, 65524, 65534, 65524, 65494, 65445, 65376, 65287, 65179, 65051, 64904, 64737, 64552, 64347, 64123, 63880, 63618, 63338, 63039, 62722, 62388, 62035, 61664, 61277, 60872, 60450, 60011, 59556, 59085, 58598, 58096, 57578, 57045, 56498, 55936, 55361, 54771, 54169, 53554, 52926, 52286, 51634, 50971, 50297, 49612, 48917, 48213, 47499, 46776, 46045, 45306, 44559, 43805, 43045, 42278, 41506, 40728, 39946, 39159, 38368, 37574, 36778, 35978, 35177, 34374, 33571, 32767, 31962, 31159, 30356, 29555, 28755, 27959, 27165, 26374, 25587, 24805, 24027, 23255, 22488, 21728, 20974, 20227, 19488, 18757, 18034, 17320, 16616, 15921, 15236, 14562, 13899, 13247, 12607, 11979, 11364, 10762, 10172, 9597, 9035, 8488, 7955, 7437, 6935, 6448, 5977, 5522, 5083, 4661, 4256, 3869, 3498, 3145, 2811, 2494, 2195, 1915, 1653, 1410, 1186, 981, 796, 629, 482, 354, 246, 157, 88, 39, 9, 0, 9, 39, 88, 157, 246, 354, 482, 629, 796, 981, 1186, 1410, 1653, 1915, 2195, 2494, 2811, 3145, 3498, 3869, 4256, 4661, 5083, 5522, 5977, 6448, 6935, 7437, 7955, 8488, 9035, 9597, 10172, 10762, 11364, 11979, 12607, 13247, 13899, 14562, 15236, 15921, 16616, 17320, 18034, 18757, 19488, 20227, 20974, 21728, 22488, 23255, 24027, 24805, 25587, 26374, 27165, 27959, 28755, 29555, 30356, 31159, 31962];

// Function to approximate sine using the lookup table
fn approx_sin(x: u64) -> u16 {
    // Map the input to the range [0, 255]
    let index = (x >> 8) as usize;
    SINE_TABLE[index%256]
}

// Function to compute the siren wave
// t is between 0 and 2000000
fn siren_wave(t: u64) -> u16 {
    let f1 = 1000; // First frequency in Hz
    let f2 = 2000; // Second frequency in Hz
    let modulation_freq = 4; // Modulation frequency in Hz

    // Convert t from milliseconds to a suitable range for the sine calculation
    let t_scaled = t as u64 * 32768 / 2000000;
    // println!("t_scaled: {}", t_scaled);
    let freq = f1 + (f2 - f1) * approx_sin(t_scaled * modulation_freq as u64) as u64 / 32768;
    // Compute the siren wave using fixed-point arithmetic
    approx_sin(t_scaled * freq)
}

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // generator version: 0.3.0

    esp_println::logger::init_logger_from_env();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(size: 72 * 1024);

        
    let timer0 = TimerGroup::new(peripherals.TIMG1);
    esp_hal_embassy::init(timer0.timer0);

    info!("Embassy initialized!");


    // initialize peripheral, not that we use 52MHz as the frequency as we target PWM at 520kHz
    // this makes the timer calculation more accurate.
    // note that the actual frequency is about 530kHz
    let clock_cfg = PeripheralClockConfig::with_frequency(Rate::from_mhz(52)).unwrap();

    let mut mcpwm = McPwm::new(peripherals.MCPWM0, clock_cfg);

    let pin = peripherals.GPIO2;
    // connect operator0 to timer0
    mcpwm.operator0.set_timer(&mcpwm.timer0);
    // // connect operator0 to pin
    let mut pwm_pin = mcpwm
        .operator0
        .with_pin_a(pin, PwmPinConfig::UP_ACTIVE_HIGH);

    // start timer with timestamp values in the range of 0..=99 and a frequency
    // of 520 kHz
    let timer_clock_cfg = clock_cfg
        .timer_clock_with_frequency(99, PwmWorkingMode::Increase, Rate::from_khz(520))
        .unwrap();
    mcpwm.timer0.start(timer_clock_cfg);

    let _ = spawner;
    let t = Instant::now();
    loop {
        let t = t.elapsed().as_micros() %2000000;
        let wave = siren_wave(t) as u32 * 50 / 32768;
        pwm_pin.set_timestamp(wave as u16);
    }

}
