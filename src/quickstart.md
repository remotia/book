# Quickstart

remotia's [crate](https://crates.io/crates/remotia) can be added to the project as any dependency: 

```toml
[dependencies.remotia]
version = "0.1.0" # Check crates.io to know the last released version
default-features = false # It is recommended to disable all features by default and enable only the needed ones
features = ["buffers", "profilation", "transmission"]
```

## Features
Each feature enables a specific part of the framework, such that only adopted components can be kept to speed up compilation. Check the [Cargo.toml](https://github.com/remotia/remotia/blob/main/Cargo.toml) for the list of available features and which crates they enable.

