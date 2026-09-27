use std::path::PathBuf;
use std::{env, fs};

/// Name, default value and whether the value must be a power of two for the index maps.
const SIZES: [(&str, usize, bool); 3] = [
    ("MAX_TASKS", 8, true),
    ("MAX_IRQS", 4, true),
    ("MAX_NESTING", 4, false),
];

fn main() {
    // Emits cfgs like `arm_architecture = "v7e-m"`.
    arm_targets::process();

    let linear_map = env::var_os("CARGO_FEATURE_LINEAR_MAP").is_some();
    let mut config = String::new();
    for (name, default, map_size) in SIZES {
        let var = format!("EMBASSY_EXECUTOR_STATS_{name}");
        println!("cargo:rerun-if-env-changed={var}");
        let value = match env::var(&var) {
            Ok(value) => value
                .parse::<usize>()
                .unwrap_or_else(|_| panic!("{var} must be a positive integer, got {value:?}")),
            Err(_) => default,
        };
        if value == 0 {
            panic!("{var} must not be zero");
        }
        if map_size && !linear_map && !value.is_power_of_two() {
            panic!("{var} must be a power of two without the `linear-map` feature, got {value}");
        }
        config.push_str(&format!("pub const {name}: usize = {value};\n"));
    }

    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    fs::write(out.join("config.rs"), config).unwrap();
}
