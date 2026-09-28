# The example apps only build for their own targets, which are set in their `.cargo/config.toml`.
# Cargo only reads that file when it runs inside the app directory, so the apps are handled
# separately from the library.

apps := "stm32h7-app stm32f0-app"

# Everything a change should pass.
all: fmt-check clippy doc build

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all --check

check: (lib "check") (app "check")

clippy: (lib "clippy" "-- -D warnings") (app "clippy -- -D warnings")

build: (app "build")

doc:
    RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --target thumbv7em-none-eabihf --features cortex-m,defmt

# Runs a cargo command on the library for all feature sets, including ARMv6-M without DWT.
lib cmd *args:
    cargo {{cmd}} --target thumbv7em-none-eabihf {{args}}
    cargo {{cmd}} --target thumbv7em-none-eabihf --features cortex-m,defmt {{args}}
    cargo {{cmd}} --target thumbv7em-none-eabihf --features cortex-m,defmt,fnv-map {{args}}
    cargo {{cmd}} --target thumbv6m-none-eabi --features cortex-m,defmt {{args}}

# Runs a cargo command inside each example app, for example `just app "build --release"`.
app args:
    for app in {{apps}}; do (cd $app && cargo {{args}}) || exit 1; done
