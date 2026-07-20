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

This example demonstrates a two-pipeline architecture where a **main** pipeline processes frames by tagging each with a unique frame_id, encoding them with a dummy codec, and checking quality against a PSNR threshold, while an **error** pipeline catches and logs any frames that fall below the threshold. The DTO carries a frame_id, a psnr score, an encoded_data buffer, and an optional error field, flowing through a tick stage, a tagging stage, an encoding stage, a quality gate, and an error switch before being routed to either pipeline.

```rust
use remotia::{
    traits::{FrameError, FrameProcessor},
    pipeline::{Pipeline, PipelineRegistry, Component},
    processors::{
        ticker::Ticker,
        error_switch::OnErrorSwitch,
    },
};
use async_trait::async_trait;
use rand::Rng;

// ── 1. DTO ── the data structure that flows through every pipeline stage

#[derive(Debug, Default)]
struct MyDto {
    frame_id: u64,
    psnr: f64,
    encoded_data: Vec<u8>,
    error: Option<String>,
}

impl FrameError<String> for MyDto {
    fn report_error(&mut self, error: String) { self.error = Some(error); }
    fn get_error(&self) -> Option<String> { self.error.clone() }
}

// ── 2. Processors ──

/// Assigns a unique frame_id on each call.
struct FrameTagger {
    current_frame_id: u64,
}

#[async_trait]
impl FrameProcessor<MyDto> for FrameTagger {
    async fn process(&mut self, mut dto: MyDto) -> Option<MyDto> {
        self.current_frame_id += 1;
        dto.frame_id = self.current_frame_id;
        Some(dto)
    }
}

/// Populates the DTO with random psnr and encoded_data values.
struct DummyCodec;

#[async_trait]
impl FrameProcessor<MyDto> for DummyCodec {
    async fn process(&mut self, mut dto: MyDto) -> Option<MyDto> {
        let mut rng = rand::thread_rng();
        dto.psnr = rng.gen_range(20.0..50.0);
        let len = rng.gen_range(100..1000);
        dto.encoded_data = (0..len).map(|_| rng.gen()).collect();
        Some(dto)
    }
}

/// Reports an error when psnr is below the configured threshold.
struct QualityGate {
    threshold: f64,
}

#[async_trait]
impl FrameProcessor<MyDto> for QualityGate {
    async fn process(&mut self, mut dto: MyDto) -> Option<MyDto> {
        if dto.psnr < self.threshold {
            dto.report_error(
                format!("psnr {:.2} below threshold {}", dto.psnr, self.threshold),
            );
        }
        println!(
            "main: frame_id = {}, psnr = {:.2}, data_len = {}, error = {}",
            dto.frame_id,
            dto.psnr,
            dto.encoded_data.len(),
            dto.error.is_some(),
        );
        Some(dto)
    }
}

/// Logs the error on a frame and discards it.
struct ErrorLogger;

#[async_trait]
impl FrameProcessor<MyDto> for ErrorLogger {
    async fn process(&mut self, dto: MyDto) -> Option<MyDto> {
        if let Some(ref err) = dto.error {
            println!("error pipeline: frame {} failed — {}", dto.frame_id, err);
        }
        None
    }
}

// ── 3. Assembly ── register error pipeline, then build main pipeline
//    with tick → increment → encode → quality gate → error switch stages

#[tokio::main]
async fn main() {
    let mut registry = PipelineRegistry::<MyDto, &str>::new();

    // Register the error pipeline first so we can feed it via OnErrorSwitch
    let error_pipeline = Pipeline::new()
        .link(Component::singleton(ErrorLogger))
        .tag("error");
    registry.register("error", error_pipeline);

    // Build an OnErrorSwitch that redirects to the error pipeline
    let error_switch = {
        let error_pipe = registry.get_mut(&"error");
        OnErrorSwitch::new(error_pipe)
    };

    // Build the main pipeline
    let main_pipeline = Pipeline::new()
        .link(
            Component::new()
                .append(Ticker::new(100))         // tick every 100ms
                .append(FrameTagger { current_frame_id: 0 })
                .append(DummyCodec)
                .append(QualityGate { threshold: 30.0 })
                .append(error_switch)              // routes only frames with errors
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
2. `FrameTagger` increments `frame_id` by 1 each tick.
3. `DummyCodec` fills `psnr` with a random value between 20.0 and 50.0 and `encoded_data` with random bytes (100–1000 bytes).
4. `QualityGate` checks whether `psnr` is below the threshold (30.0); if so, it reports an error on the DTO. It always passes the frame forward.
5. `OnErrorSwitch` inspects the DTO's error field: if an error is present it feeds the frame into the error pipeline; otherwise the frame is consumed in the main pipeline.
6. The error pipeline logs the failure with `ErrorLogger` and discards the frame.

## Next steps

- Read the [Key concepts](./key_concepts.md) for the full trait and API reference.
- Browse the [Processors](./processors.md) catalog to find built-in processors for your pipeline.
- See [Pipelines & lifecycle](./pipeline-lifecycle.md) for shutdown, feedable pipelines, and the registry.
- Check the [Crate map](./crate-map.md) for feature flags and optional crates.
