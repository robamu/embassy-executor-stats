//! Run time statistics on the STM32F051, set up for the STM32F0DISCOVERY board.
//!
//! The Cortex-M0 has no DWT cycle counter, so the 32-bit TIM2 provides the counter instead.
#![no_std]
#![no_main]

use embassy_executor::{SpawnToken, Spawner};
use embassy_executor_stats::Counter;
use embassy_executor_stats::cortex_m::interrupt_scope;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::interrupt;
use embassy_stm32::interrupt::InterruptExt;
use embassy_stm32::peripherals::TIM2;
use embassy_stm32::{Peri, pac, rcc, timer};
use embassy_time::{Duration, Instant, Ticker, Timer};
use {defmt_rtt as _, panic_probe as _};

const STATS_PERIOD: Duration = Duration::from_secs(5);

/// TIM2 counting up with its reset configuration: no prescaler and the full 32-bit range.
///
/// The counter only reads the count register, so it needs no timer instance. The TIM must not be
/// used for anything else.
struct TimCounter;

impl TimCounter {
    fn start(tim: Peri<'static, TIM2>) {
        let timer = timer::low_level::Timer::new(tim);
        timer.start();
        // Dropping the timer would disable its clock.
        core::mem::forget(timer);
    }
}

impl Counter for TimCounter {
    fn now() -> u32 {
        pac::TIM2.cnt().read()
    }

    fn frequency_hz() -> u32 {
        rcc::frequency::<TIM2>().0
    }
}

/// Plain interrupt handler without an executor, pended by [`busy_worker`].
#[interrupt]
fn USART1() {
    let _scope = interrupt_scope("USART1");
    cortex_m::asm::delay(2_000);
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    TimCounter::start(p.TIM2);
    embassy_executor_stats::init::<TimCounter>();

    // SAFETY: The USART1 handler only burns cycles and uses no shared state.
    unsafe { interrupt::USART1.enable() };

    // User LEDs of the STM32F0DISCOVERY.
    let blue = Output::new(p.PC8, Level::Low, Speed::Low);
    let green = Output::new(p.PC9, Level::Low, Speed::Low);
    spawner.spawn(named(
        blink(blue, Duration::from_millis(250)).unwrap(),
        "led_blue",
    ));
    spawner.spawn(named(
        blink(green, Duration::from_millis(500)).unwrap(),
        "led_green",
    ));

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

#[embassy_executor::task(pool_size = 2)]
async fn blink(mut led: Output<'static>, period: Duration) {
    let mut ticker = Ticker::every(period);
    loop {
        led.toggle();
        ticker.next().await;
    }
}

/// Burns a fixed amount of cycles periodically and pends the USART1 interrupt.
#[embassy_executor::task]
async fn busy_worker() {
    loop {
        cortex_m::asm::delay(20_000);
        interrupt::USART1.pend();
        Timer::after_millis(10).await;
    }
}

/// Runs once and ends, so the stats show a finished task whose slot is reused.
#[embassy_executor::task]
async fn short_lived() {
    for _ in 0..5 {
        cortex_m::asm::delay(40_000);
        Timer::after_millis(100).await;
    }
}
