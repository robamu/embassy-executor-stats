//! Run time statistics on the Vorago VA108xx, set up for the REB1 board.
//!
//! The Cortex-M0 has no DWT cycle counter, so a TIM peripheral provides the counter instead.
#![no_std]
#![no_main]

use embassy_executor::{SpawnToken, Spawner};
use embassy_executor_stats::Counter;
use embassy_executor_stats::cortex_m::interrupt_scope;
use embassy_time::{Duration, Instant, Ticker, Timer};
use va108xx_hal::gpio::{Output, PinState};
use va108xx_hal::pac::{self, interrupt};
use va108xx_hal::pins::PinsA;
use va108xx_hal::time::Hertz;
use va108xx_hal::timer::CountdownTimer;
use {defmt_rtt as _, panic_probe as _};

const SYSCLK: Hertz = Hertz::from_raw(50_000_000);
const STATS_PERIOD: Duration = Duration::from_secs(5);

/// TIM21 counting down from `u32::MAX` at the system clock.
///
/// The counter only reads the count register, so it needs no timer instance. The TIM must not be
/// used for anything else.
struct TimCounter;

impl TimCounter {
    fn start(tim: pac::Tim21) {
        let mut timer = CountdownTimer::new(tim, SYSCLK);
        timer.set_reload(u32::MAX);
        timer.set_count(u32::MAX);
        timer.enable();
        // Dropping the timer keeps it running.
    }
}

impl Counter for TimCounter {
    fn now() -> u32 {
        // SAFETY: Reading the count register has no side effects.
        let count = unsafe { (*pac::Tim21::ptr()).cnt_value().read().bits() };
        // The timer counts down, the statistics need a counter which counts up.
        u32::MAX - count
    }

    fn hz() -> u32 {
        SYSCLK.to_raw()
    }
}

/// Plain interrupt handler without an executor, pended by [`busy_worker`].
#[interrupt]
fn OC15() {
    let _scope = interrupt_scope("OC15");
    cortex_m::asm::delay(5_000);
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let dp = pac::Peripherals::take().unwrap();

    va108xx_hal::embassy_time::init(dp.tim23, dp.tim22, SYSCLK);
    TimCounter::start(dp.tim21);
    embassy_executor_stats::init::<TimCounter>();

    // SAFETY: The OC15 handler only burns cycles and uses no shared state.
    unsafe { cortex_m::peripheral::NVIC::unmask(pac::Interrupt::OC15) };

    // User LEDs of the REB1. They are on when the pin is low.
    let pins = PinsA::new(dp.porta);
    let led_d2 = Output::new(pins.pa10, PinState::High);
    let led_d3 = Output::new(pins.pa7, PinState::High);
    let led_d4 = Output::new(pins.pa6, PinState::High);
    spawner.spawn(named(blink(led_d2, Duration::from_millis(250)).unwrap(), "led_d2"));
    spawner.spawn(named(blink(led_d3, Duration::from_millis(500)).unwrap(), "led_d3"));
    spawner.spawn(named(blink(led_d4, Duration::from_millis(1000)).unwrap(), "led_d4"));

    spawner.spawn(named(busy_worker().unwrap(), "busy_worker"));

    loop {
        match short_lived() {
            Ok(token) => spawner.spawn(named(token, "short_lived")),
            Err(_) => defmt::warn!("short_lived is still running"),
        }
        Timer::after(STATS_PERIOD).await;
        defmt::println!("");
        defmt::println!("==================== Task stats ====================");
        embassy_executor_stats::print_stats(Instant::now().as_micros());
    }
}

fn named<S>(token: SpawnToken<S>, name: &'static str) -> SpawnToken<S> {
    token.metadata().set_name(name);
    token
}

#[embassy_executor::task(pool_size = 3)]
async fn blink(mut led: Output, period: Duration) {
    let mut ticker = Ticker::every(period);
    loop {
        led.toggle();
        ticker.next().await;
    }
}

/// Burns a fixed amount of cycles periodically and pends the OC15 interrupt.
#[embassy_executor::task]
async fn busy_worker() {
    loop {
        cortex_m::asm::delay(50_000);
        cortex_m::peripheral::NVIC::pend(pac::Interrupt::OC15);
        Timer::after_millis(10).await;
    }
}

/// Runs once and ends, so the stats show a finished task whose slot is reused.
#[embassy_executor::task]
async fn short_lived() {
    for _ in 0..5 {
        cortex_m::asm::delay(100_000);
        Timer::after_millis(100).await;
    }
}
