# Platform-dependant screen snapper — Handling platform-specific backends

Same capture-and-save pipeline as the [screen snapper](./screen_snapper.md), but uses Cargo feature flags to select between X11/XCap and Wayland/libwayshot capture backends at compile time.

Full source: [GitHub — platform-dependant-screen-snapper](https://github.com/remotia/examples/tree/main/platform-dependant-screen-snapper)

## Architecture

Identical runtime layout — two components in one pipeline. The difference is in how the capturer is selected:

```
Component 1: Capture                          Component 2: Save
┌──────────────────────────────────┐          ┌─────────────────┐
│ Ticker → BufferAllocator →       │  channel │ PNGBufferSaver  │
│ [xcap OR wayshot] Capturer       │─────────▶│                 │
└──────────────────────────────────┘          └─────────────────┘
```

## Feature flags in Cargo.toml

```toml
[features]
default = ["wayshot"]
xcap = ["dep:xcap"]
wayshot = ["dep:libwayshot"]
```

The `lib.rs` uses compile-time guards to prevent misconfiguration:

```rust
#[cfg(not(any(feature = "xcap", feature = "wayshot")))]
compile_error!("No snapper backend enabled");

#[cfg(all(feature = "xcap", feature = "wayshot"))]
compile_error!("Compiling with both wayshot and xcap is not currently supported.");

#[cfg(feature = "wayshot")]
pub mod wayshot_capturer;

#[cfg(feature = "xcap")]
pub mod xcap_capturer;
```

## Runtime selection with conditional compilation

A `capture.rs` module provides the same interface regardless of backend:

```rust
#[cfg(feature = "xcap")]
pub mod xcap {
    pub fn fetch_screen_resolution() -> (u32, u32) {
        xcap_utils::display_size(MONITOR_ID)
    }
    pub fn capturer_processor() -> XCapCapturer<Buffers> {
        XCapCapturer::builder()
            .buffer_key(Buffers::CapturedScreenBuffer)
            .monitor_id(MONITOR_ID)
            .build()
    }
}
#[cfg(feature = "xcap")]
pub use xcap::*;

#[cfg(feature = "wayshot")]
pub mod libwayshot {
    pub fn fetch_screen_resolution() -> (u32, u32) {
        wayshot_utils::display_size()
    }
    pub fn capturer_processor() -> WayshotCapturer<Buffers> {
        WayshotCapturer::builder()
            .buffer_key(Buffers::CapturedScreenBuffer)
            .build()
    }
}
#[cfg(feature = "wayshot")]
pub use libwayshot::*;
```

The `main.rs` calls `fetch_screen_resolution()` and `capturer_processor()` without knowing which backend is active.

## Building

```bash
# Default (wayshot)
cargo run --example autosnapper

# X11/XCap backend
cargo run --example autosnapper --no-default-features --features xcap
```

## Key takeaways

- Cargo feature flags are the idiomatic Rust way to handle platform-specific dependencies. Remotia's `FrameProcessor` trait makes it easy — both backends implement the same trait, so the pipeline code is backend-agnostic.
- `compile_error!` guards prevent invalid feature combinations at compile time.
- The DTO and save component are identical across backends — only the capture processor changes.
