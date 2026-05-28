# Quickstart

## Add the dependency

```toml
[dependencies.remotia]
version = "0.1.0" # Check crates.io for the latest version
default-features = false
features = []     # Enable only what you need (see Crate map)
```

> **Note:** The feature flag is named `profilation` for historical reasons, but it enables profiling utilities.

## A minimal pipeline

This example creates a two-pipeline architecture: a **main** pipeline that ticks, increments a counter, and prints it, and an **overflow** pipeline that handles frames when the counter exceeds a threshold.

```rust
use remotia::{
    traits::{FrameError, FrameProcessor},
    pipeline::{Pipeline, PipelineRegistry},
    processors::{
        ticker::Ticker,
        switch::Switch,
        functional::Closure,
    },
};
use async_trait::async_trait;

// ── 1. Define the DTO ────────────────────────────────────────────────

#[derive(Debug, Default)]
struct MyDto {
    counter: u64,
    error: Option<String>,
}

impl FrameError<String> for MyDto {
    fn report_error(&mut self, error: String) { self.error = Some(error); }
    fn get_error(&self) -> Option<String> { self.error.clone() }
}

// ── 2. Define a custom processor ─────────────────────────────────────

struct Incrementer;

#[async_trait]
impl FrameProcessor<MyDto> for Incrementer {
    async fn process(&mut self, mut dto: MyDto) -> Option<MyDto> {
        dto.counter += 1;
        Some(dto)
    }
}

// ── 3. Build and run the pipelines ───────────────────────────────────

#[tokio::main]
async fn main() {
    let mut registry = PipelineRegistry::<MyDto, &str>::new();

    // Register the overflow pipeline first so we can get a feeder for the switch
    let overflow_pipeline = Pipeline::new()
        .link(
            Component::singleton(
                Closure::new(|dto: MyDto| {
                    println!("overflow: counter = {}", dto.counter);
                    None // consumed — pipeline stops processing this frame
                })
            )
        )
        .tag("overflow");
    registry.register("overflow", overflow_pipeline);

    // Build a switch that redirects to the overflow pipeline
    let overflow_switch = {
        let overflow_pipe = registry.get_mut(&"overflow");
        Switch::new(overflow_pipe)
    };

    // Build the main pipeline
    let main_pipeline = Pipeline::new()
        .link(
            Component::new()
                .append(Ticker::new(100))     // tick every 100ms
                .append(Incrementer)           // increment counter
                .append(Closure::new(|dto: MyDto| {  // print & check
                    println!("main: counter = {}", dto.counter);
                    if dto.counter > 10 {
                        None // send to overflow via switch below
                    } else {
                        Some(dto)
                    }
                }))
                .append(overflow_switch)       // redirect if previous returned None
                .tag("main-step")
        )
        .tag("main");

    registry.register("main", main_pipeline);

    // Run all pipelines (blocks until all tasks complete)
    registry.run().await;
}
```

### What happens at runtime

1. The `Ticker` paces the loop at ~10 Hz.
2. `Incrementer` bumps `counter` by 1 each tick.
3. The closure prints the counter. If `counter > 10`, it returns `None`.
4. Returning `None` means the DTO does **not** reach the `Switch`. If you want the switch to fire *instead* of consuming the frame, restructure so the switch comes before the conditional return. A typical pattern is:

```rust
Component::new()
    .append(Ticker::new(100))
    .append(Incrementer)
    .append(Closure::new(|dto: MyDto| {
        println!("main: counter = {}", dto.counter);
        Some(dto) // always pass forward
    }))
    // The switch always redirects — frame never continues past it
```

To conditionally redirect, use `OnErrorSwitch` or implement the branching logic inside a single closure.

## Next steps

- Read the [Key concepts](./key_concepts.md) for the full trait and API reference.
- Browse the [Processors](./processors.md) catalog to find built-in processors for your pipeline.
- See [Pipelines & lifecycle](./pipeline-lifecycle.md) for shutdown, feedable pipelines, and the registry.
- Check the [Crate map](./crate-map.md) for feature flags and optional crates.
