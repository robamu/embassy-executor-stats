#![no_std]
#![no_main]

use embassy_executor::{InterruptExecutor, SpawnToken, Spawner};
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::interrupt;
use embassy_stm32::interrupt::{InterruptExt, Priority};
use embassy_time::{Duration, Instant, Ticker, Timer};
use {defmt_rtt as _, panic_probe as _};

const STATS_PERIOD: Duration = Duration::from_secs(5);

static EXECUTOR_HIGH: InterruptExecutor = InterruptExecutor::new();

#[interrupt]
unsafe fn UART4() {
    unsafe { EXECUTOR_HIGH.on_interrupt() }
}

/// Plain interrupt handler without an executor, pended by [`high_prio`] which it preempts.
#[interrupt]
fn UART5() {
    let _scope = embassy_executor_stats::cortex_m::interrupt_scope("UART5");
    cortex_m::asm::delay(5_000);
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    let mut core = cortex_m::Peripherals::take().unwrap();
    // Assumes the default D1CPRE divider of 1, so the core runs at sysclk.
    let core_clock = embassy_stm32::rcc::clocks(&p.RCC).sys.to_hertz().unwrap();
    embassy_executor_stats::cortex_m::init(&mut core.DCB, &mut core.DWT, core_clock.0);

    interrupt::UART5.set_priority(Priority::P5);
    // SAFETY: The UART5 handler only burns cycles and uses no shared state.
    unsafe { interrupt::UART5.enable() };

    interrupt::UART4.set_priority(Priority::P6);
    let high_spawner = EXECUTOR_HIGH.start(interrupt::UART4);
    high_spawner.spawn(named(high_prio().unwrap(), "high_prio"));

    // User LEDs of the Nucleo-H753ZI.
    let green = Output::new(p.PB0, Level::Low, Speed::Low);
    let yellow = Output::new(p.PE1, Level::Low, Speed::Low);
    let red = Output::new(p.PB14, Level::Low, Speed::Low);
    spawner.spawn(named(blink(green, Duration::from_millis(250)).unwrap(), "led_green"));
    spawner.spawn(named(blink(yellow, Duration::from_millis(500)).unwrap(), "led_yellow"));
    spawner.spawn(named(blink(red, Duration::from_millis(1000)).unwrap(), "led_red"));

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
async fn blink(mut led: Output<'static>, period: Duration) {
    let mut ticker = Ticker::every(period);
    loop {
        led.toggle();
        ticker.next().await;
    }
}

/// Burns a fixed amount of cycles periodically to have something to measure.
#[embassy_executor::task]
async fn busy_worker() {
    loop {
        cortex_m::asm::delay(100_000);
        Timer::after_millis(10).await;
    }
}

/// Runs once and ends, so the stats show a finished task whose slot is reused.
#[embassy_executor::task]
async fn short_lived() {
    for _ in 0..5 {
        cortex_m::asm::delay(200_000);
        Timer::after_millis(100).await;
    }
}

/// Runs on the interrupt executor and preempts the thread-mode tasks.
#[embassy_executor::task]
async fn high_prio() {
    let mut ticker = Ticker::every(Duration::from_millis(5));
    loop {
        cortex_m::asm::delay(10_000);
        interrupt::UART5.pend();
        cortex_m::asm::delay(10_000);
        ticker.next().await;
    }
}
