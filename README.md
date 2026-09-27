# Run-time statistics for applications using `embassy-executor`

This crate provides information similar to the `vTaskGetRunTimeStats` of FreeRTOS.

The library uses the `trace` feature of `embassy-executor` to measure the time spent in tasks,
executors and interrupt handlers. It prints a table like this:

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

- `cortex-m`: Uses the DWT cycle counter and identifies interrupts through `VECTACTIVE`.
- `defmt`: Printers for the statistics.
- `linear-map`: Stores the statistics in a `heapless::LinearMap`, which needs less code and
  memory for small sizes.

On other architectures, implement the `Counter` trait with any free-running counter.

## Examples

- [`stm32h7-app`](stm32h7-app): STM32H753 (Cortex-M7) with a thread mode and an interrupt executor.
- [`stm32f0-app`](stm32f0-app): STM32F0DISCOVERY board with a STM32F051 (Cortex-M0). TIM2 is the
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
