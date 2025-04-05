# Basic usage

Currently, remotia can be added to the project as a git dependency:

```toml
[dependencies.remotia]
git = "https://github.com/remotia/remotia"
branch = "main"
default-features = false
features = ["buffers", "profilation", "transmission"]
```

## Features
Each feature enables a specific part of the framework, such that only adopted components can be kept to speed up compilation. Check the [Cargo.toml](https://github.com/remotia/remotia/blob/main/Cargo.toml) for the list of available features and which crates they enable.

