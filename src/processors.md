# Processors

Processors are the building blocks of a Remotia pipeline. Each processor performs one atomic operation on the frame data by implementing the `FrameProcessor<F>` trait with a single `process` method. The method takes ownership of a DTO and returns `Option<F>`: `Some(dto)` passes the data forward to the next processor, while `None` signals that processing for that frame is complete — it may have been redirected to another pipeline, stored, or dropped. This simple contract makes it easy to add, remove, or reorder processing steps without affecting the rest of the system. See the [processors module documentation](https://docs.rs/remotia/latest/remotia/processors/index.html) for the full API reference.

Processors implement the `FrameProcessor<F>` trait and perform a single atomic operation on the DTO. This page catalogs every built-in processor in `remotia-core`.

```rust
#[async_trait]
pub trait FrameProcessor<F> {
    async fn process(&mut self, frame_data: F) -> Option<F>;
}
```

---

## Routing switches

Switches move DTOs between pipelines. They return `None` (or `Some` with the original) so the current pipeline can continue or terminate for that frame.

### Switch

Redirects the DTO into a different pipeline. Returns `None` — the current pipeline stops processing this frame.

```rust
use remotia::processors::switch::Switch;

let switch = Switch::new(&mut destination_pipeline);
```

**Use when:** branching — e.g. sending frames down an error or debug pipeline instead of the main one.

---

### CloneSwitch

Clones the DTO, sends the clone to another pipeline, and passes the original forward. The DTO type must implement `Clone`.

```rust
use remotia::processors::clone_switch::CloneSwitch;

let clone_switch = CloneSwitch::new(&mut profiling_pipeline);
```

**Use when:** parallel side-channels — e.g. a profiling pipeline that receives every frame while the main pipeline continues uninterrupted.

---

### OnErrorSwitch

Redirects the DTO to a destination pipeline if it carries a matching error. If no error is present (or the error does not match), the frame passes through with `Some(frame)`. The DTO must implement `FrameError<E>`.

```rust
use remotia::processors::error_switch::OnErrorSwitch;

let error_switch = OnErrorSwitch::new::<MyDto, MyError>(&mut error_pipeline)
    .detect(MyError::Timeout)
    .detect(MyError::ConnectionError);
```

**Use when:** conditional error routing — only certain error variants are diverted.

---

### PoolingSwitch

Picks a random destination from a registered pool, stamps the DTO with the chosen pool key (via `FrameProperties`), and sends it. Returns `None`. The DTO must implement `FrameProperties<P, K>`.

```rust
use remotia::processors::pool_switch::PoolingSwitch;

let pool_switch = PoolingSwitch::<MyDto, &str, usize>::new("worker_id")
    .entry(0, &mut worker_pipeline_0)
    .entry(1, &mut worker_pipeline_1)
    .entry(2, &mut worker_pipeline_2);
```

**Use when:** fan-out to a pool of workers — e.g. distributing encoding across N encoder pipelines.

---

### DepoolingSwitch

Routes the DTO to the destination matching its pool key (read via `FrameProperties`). Returns `None`. The DTO must implement `FrameProperties<P, K>`.

```rust
use remotia::processors::pool_switch::DepoolingSwitch;

let depool_switch = DepoolingSwitch::<MyDto, &str, usize>::new("worker_id")
    .entry(0, &mut merger_pipeline_0)
    .entry(1, &mut merger_pipeline_1);
```

**Use when:** fan-in — merging results from a worker pool back into per-worker downstream pipelines.

---

## Timing

### Ticker

Waits for the configured interval, then passes the DTO forward unchanged.

```rust
use remotia::processors::ticker::Ticker;

let ticker = Ticker::new(16); // 16 ms ≈ 60 FPS
```

**Use when:** frame-rate pacing at the head of a capture pipeline, or throttling any component.

---

## Inline processors

These wrap bare functions, closures, or async functions as `FrameProcessor` implementors. Use them for quick logic without defining a dedicated struct.

### Function

Wraps a function pointer.

```rust
use remotia::processors::functional::Function;

let proc = Function::new(|mut dto: MyDto| -> Option<MyDto> {
    dto.frame_id += 1;
    Some(dto)
});
```

---

### Closure

Wraps a capturing closure.

```rust
use remotia::processors::functional::Closure;

let counter = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));
let counter_clone = counter.clone();

let proc = Closure::new(move |dto: MyDto| -> Option<MyDto> {
    counter_clone.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    Some(dto)
});
```

Convenience method on `Component`:

```rust
component.closure(|dto: MyDto| Some(dto))
```

---

### AsyncFunction

Wraps an async function pointer. Use the `async_func!` macro to create the pinned future.

```rust
use remotia::processors::async_functional::AsyncFunction;
use remotia::async_func;

async fn fetch_frame(dto: MyDto) -> Option<MyDto> {
    // async I/O ...
    Some(dto)
}

let proc = AsyncFunction::new(|dto| async_func!(async move {
    fetch_frame(dto).await
}));
```

---

## Containers

### Sequential

Runs a sequence of processors in order inside a single component. Each processor receives the output of the previous one. If any processor returns `None`, the sequence stops.

```rust
use remotia::processors::containers::sequential::Sequential;

let seq = Sequential::new()
    .append(processor_a)
    .append(processor_b)
    .append(processor_c);
```

**Use when:** grouping processors that must run in the same async task — e.g. a tick-then-capture pattern, or a sequence of buffer operations that should not be split across component boundaries.

---

## Profilation

The profilation utilities (behind the `profilation` feature flag, in the separate `remotia-profilation-utils` crate) collect execution statistics during frame processing. Remotia does not force a specific method to collect statistics, but provides profilation processors that are injected into components at the points where the phase to profile begins or ends, so that collection points are equal between executions and reproducibility is strengthened.

### Timestamp pairs

A `TimestampAdder("encoding_start")` processor at the start of the phase saves the current time into a DTO field. A time-diff processor at the end writes the elapsed time into another DTO field:

```rust
use remotia::profilation::{TimestampAdder, TimestampDiffCalculator};

Pipeline::new()
    .link(
        Component::new()
            .append(TimestampAdder("encoding_start"))
            .append(AV1Preprocessor())
            .append(AV1Compressor())
            .append(AV1Serializer())
            .append(TimestampDiffCalculator("encoding_start", "encoding_time"))
    )
    .run();
```

The same measurement works when the phase is split across multiple components, making it simple to verify whether asynchronous preprocessing improves performance:

```rust
Pipeline::new()
    .link(
        Component::new()
            .append(TimestampAdder("encoding_start"))
            .append(AV1Preprocessor())
    )
    .link(
        Component::new()
            .append(AV1Compressor())
            .append(AV1Serializer())
            .append(TimestampDiffCalculator("encoding_start", "encoding_time"))
    )
    .run();
```

### ProfiledSequential

When only the overall processing time matters and the computation runs in a single thread, `ProfiledSequential` wraps a sequence of processors and measures their total execution time:

```rust
use remotia::profilation::ProfiledSequential;

Component::new()
    .append(ProfiledSequential("encoding_time"))
    .append(AV1Preprocessor())
    .append(AV1Compressor())
    .append(AV1Serializer());
```

It can later be replaced by a plain [`Sequential`](#sequential) (which runs the same sequence without measuring) with minimal edits.

### Frame droppers and loggers

The profilation feature also includes utilities for dropping stale frames and logging statistics:

| Type | Purpose |
|---|---|
| `ThresholdBasedFrameDropper` | Drops frames exceeding a configurable delay threshold |
| `TimestampBasedFrameDropper` | Drops frames based on timestamp comparisons |
| `ConsoleAverageStatsLogger` | Logs average statistics to the console at intervals |
| `CSVFrameDataSerializer` | Serializes frame data to CSV for post-hoc analysis |
| `ConsoleDropReasonLogger` | Logs the reason for each dropped frame |

The delay threshold used by frame droppers is grounded in the paper's experimental setup, where a threshold on delay is added before rendering on the client screen to drop stale frames.

Browse the [API documentation](https://docs.rs/remotia/latest/remotia/) for the full profilation module reference.
