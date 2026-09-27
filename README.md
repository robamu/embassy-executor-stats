# Run time statistics for applications using `embassy-executor`

This crate provides information similar to [`vTaskGetRunTimeStats`][freertos-stats] of FreeRTOS.

The library uses the [`trace` feature][executor-features] of
[`embassy-executor`][embassy-executor] to measure the time spent in tasks, executors and interrupt
handlers. The results can be printed as a table:

```text
Task                Time [us]       %      Polls
------------------------------------------------
main                      812    0.01          2
busy_worker           1530000   30.59        469
short_lived             15625    0.31          5 (ended)

Interrupt           Time [us]       %      Calls
------------------------------------------------
UART5                   78125    1.56       1000

Summary             Time [us]       %
------------------------------------------------
tasks                 1797641   35.95
interrupts              78125    1.56
executor                 9870    0.19
idle + other          3114487   62.28
------------------------------------------------
uptime                5000123
```

## Features

- `cortex-m`: Provides the DWT cycle counter where the core has one, and identifies interrupts
  through `VECTACTIVE`.
- `defmt`: Printers for the statistics using [`defmt`][defmt].
- `linear-map`: Stores the statistics in a [`heapless::LinearMap`][linear-map], which needs less
  code and memory for small sizes.

On other architectures, implement the `Counter` trait with any free-running counter.

## Configuration

The sizes of the static storage are read from environment variables at build time. Set them in
the `.cargo/config.toml` of your application:

```toml
[env]
EMBASSY_EXECUTOR_STATS_MAX_TASKS = "16"   # default 8
EMBASSY_EXECUTOR_STATS_MAX_IRQS = "8"     # default 4
EMBASSY_EXECUTOR_STATS_MAX_NESTING = "8"  # default 4
```

Without the `linear-map` feature, `MAX_TASKS` and `MAX_IRQS` must be powers of two.

## Examples

- [`stm32h7-app`](stm32h7-app): STM32H753 (Cortex-M7) with a thread mode and an interrupt executor.
- [`stm32f0-app`](stm32f0-app): STM32F0DISCOVERY board with an STM32F051 (Cortex-M0). TIM2 is the
  counter, because the Cortex-M0 has no DWT cycle counter.

Run them with `cargo run` from their directory. This requires
[probe-rs](https://probe.rs).

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without
any additional terms or conditions.

[freertos-stats]: https://www.freertos.org/Documentation/02-Kernel/02-Kernel-features/08-Run-time-statistics
[embassy-executor]: https://docs.rs/embassy-executor
[executor-features]: https://docs.rs/crate/embassy-executor/latest/features
[defmt]: https://docs.rs/defmt
[linear-map]: https://docs.rs/heapless/0.9/heapless/linear_map/type.LinearMap.html
