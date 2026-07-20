# Quickstart

This example demonstrates a two-pipeline architecture: a **main** pipeline processes frames by tagging each with a unique `frame_id`, encoding them with a dummy codec, and checking quality against a PSNR threshold, while an **error** pipeline catches and logs any frames that fall below the threshold.

Full source: [code-samples/examples/quickstart.rs](https://github.com/remotia/book/blob/main/code-samples/examples/quickstart.rs)

### 1. Data Transfer Object (DTO)

The DTO carries a `frame_id`, a `psnr` score, an `encoded_data` buffer, and an optional `error` field. It implements `FrameError<String>` so pipeline stages can report errors on it.

```rust
use remotia::{
    traits::{FrameError, FrameProcessor},
    pipeline::{Pipeline, component::Component, registry::PipelineRegistry},
    processors::{
        ticker::Ticker,
        error_switch::OnErrorSwitch,
    },
};
use async_trait::async_trait;
use rand::Rng;

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
```

### 2. Pipeline Processors

Each processor implements `FrameProcessor<MyDto>` and transforms or inspects the DTO as it moves through the pipeline.

**FrameTagger** — assigns a unique frame ID on each invocation:

```rust
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
```

**DummyCodec** — populates the DTO with random PSNR and encoded data:

```rust
struct DummyCodec;

#[async_trait]
impl FrameProcessor<MyDto> for DummyCodec {
    async fn process(&mut self, mut dto: MyDto) -> Option<MyDto> {
        let mut rng = rand::thread_rng();
        dto.psnr = rng.gen_range(20.0..50.0);
        let len = rng.gen_range(100..1000);
        dto.encoded_data = (0..len).map(|_| rng.r#gen()).collect();
        Some(dto)
    }
}
```

**QualityGate** — checks whether the PSNR is below a threshold and reports an error if so:

```rust
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
```

**ErrorLogger** — logs any error on a frame and discards it:

```rust
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
```

### 3. Pipeline Assembly

Register the error pipeline first, making it **feedable** so `OnErrorSwitch` can push frames into it. Then build the main pipeline with a tick rate-limiter, frame tagger, dummy codec, quality gate, and error switch. Finally register and run everything:

```rust
#[tokio::main]
async fn main() {
    let mut registry = PipelineRegistry::<MyDto, &str>::new();

    let error_pipeline = Pipeline::new()
        .link(Component::singleton(ErrorLogger))
        .feedable()
        .tag("error");
    registry.register("error", error_pipeline);

    let error_switch = {
        let error_pipe = registry.get_mut(&"error");
        OnErrorSwitch::new(error_pipe)
    };

    let main_pipeline = Pipeline::new()
        .link(
            Component::new()
                .append(Ticker::new(100))
                .append(FrameTagger { current_frame_id: 0 })
                .append(DummyCodec)
                .append(QualityGate { threshold: 30.0 })
                .append(error_switch)
                .tag("main-step")
        )
        .tag("main");

    registry.register("main", main_pipeline);

    registry.run().await;
}
```

### What happens at runtime

1. The `Ticker` paces the loop at ~10 Hz.
2. `FrameTagger` increments `frame_id` by 1 each tick.
3. `DummyCodec` fills `psnr` with a random value between 20.0 and 50.0 and `encoded_data` with random bytes (100–1000 bytes).
4. `QualityGate` checks whether `psnr` is below the threshold (30.0); if so, it reports an error on the DTO. It always passes the frame forward.
5. `OnErrorSwitch` inspects the DTO's error field and splits execution into two branches:
   - **Main branch** — no error is present: the frame is consumed within the main pipeline.
   - **Error branch** — an error is present: the frame is forwarded to the registered error pipeline.
6. In the error pipeline, `ErrorLogger` prints the failure details and the frame is discarded (returns `None`).

## Next steps

- Read the [Key concepts](./key_concepts.md) for the full trait and API reference.
- Browse the [Processors](./processors.md) catalog to find built-in processors for your pipeline.
- See [Pipelines & lifecycle](./pipeline-lifecycle.md) for shutdown, feedable pipelines, and the registry.
- Check the [Crate map](./crate-map.md) for feature flags and optional crates.
