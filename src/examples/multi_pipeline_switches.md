# Multi-pipeline with switches — Connecting pipelines

This example demonstrates one of remotia's key mechanisms: multiple pipelines connected by switch processors, which route frames between them. A main processing pipeline routes frames to different downstream pipelines based on conditions.

Full source: [GitHub — examples](https://github.com/remotia/examples)

## Architecture

Three pipelines connected by switches:

```
Pipeline "main"
┌──────────────────────────────────────────┐
│ Component:                               │
│  Ticker → Closure(print) → CloneSwitch ──┼──▶ Pipeline "profiling"
│                           → Switch ──────┼──▶ Pipeline "errors" (if error)
└──────────────────────────────────────────┘
```

- **Main pipeline**: generates frames, prints their state, clones each frame to the profiling pipeline, and conditionally switches errored frames to the error pipeline.
- **Profiling pipeline**: receives a clone of every frame and logs statistics.
- **Error pipeline**: receives only frames that carry an error.

## The DTO

```rust
#[derive(Debug, Default, Clone)]
struct FrameDto {
    frame_id: u64,
    timestamp: Option<u128>,
    error: Option<String>,
}

impl FrameProperties<String, u64> for FrameDto {
    fn set(&mut self, key: String, value: u64) {
        if key == "frame_id" { self.frame_id = value; }
    }
    fn get(&self, key: &String) -> Option<u64> {
        if key == "frame_id" { Some(self.frame_id) } else { None }
    }
}

impl FrameError<String> for FrameDto {
    fn report_error(&mut self, error: String) { self.error = Some(error); }
    fn get_error(&self) -> Option<String> { self.error.clone() }
}
```

Note that `Clone` is required because `CloneSwitch` clones the DTO.

## Building the pipelines

```rust
let mut registry = PipelineRegistry::<FrameDto, &str>::new();

// 1. Register profiling pipeline (receives a clone of every frame)
let profiling_pipeline = Pipeline::new()
    .link(Component::singleton(Closure::new(|dto: FrameDto| {
        log::info!("[profiling] frame_id={}", dto.frame_id);
        None
    })))
    .tag("profiling");
registry.register("profiling", profiling_pipeline);

// 2. Register error pipeline (receives only errored frames)
let error_pipeline = Pipeline::new()
    .link(Component::singleton(Closure::new(|dto: FrameDto| {
        log::warn!("[errors] frame_id={} error={:?}", dto.frame_id, dto.error);
        None
    })))
    .tag("errors");
registry.register("errors", error_pipeline);

// 3. Build switches that target the above pipelines
let clone_switch = CloneSwitch::new(registry.get_mut(&"profiling"));
let error_switch = OnErrorSwitch::new(registry.get_mut(&"errors"));

// 4. Build main pipeline
let main_pipeline = Pipeline::new()
    .link(
        Component::new()
            .append(Ticker::new(33))       // ~30 FPS
            .append(Closure::new(|mut dto: FrameDto| {
                dto.frame_id += 1;
                log::info!("[main] frame_id={}", dto.frame_id);
                Some(dto)
            }))
            .append(error_switch)           // redirect errored frames
            .append(clone_switch)           // clone to profiling pipeline
    )
    .tag("main");
registry.register("main", main_pipeline);

registry.run().await;
```

## How switches compose

The processor order matters:

1. `error_switch` checks if the frame has an error. If yes, it sends the frame to the error pipeline and returns `None` — the frame never reaches `clone_switch`. If no error, it returns `Some(frame)` and processing continues.
2. `clone_switch` clones the frame, sends the clone to profiling, and returns `Some(original)`. The original then exits the component and flows to the next component in the main pipeline (if any).

By placing `error_switch` before `clone_switch`, errored frames are *not* cloned to profiling. Reverse the order if you want profiling to see every frame including errored ones.

## Key takeaways

- **`CloneSwitch`** is non-destructive: it forwards the original and sends a copy elsewhere. Use it for side-channels like profiling or logging.
- **`OnErrorSwitch`** is conditional: it only redirects when the DTO carries a matching error. Otherwise the frame passes through.
- **`Switch`** is unconditional: it always redirects and returns `None`. Use it when the main pipeline should not continue for that frame.
- **Processor ordering** within a component determines which switches fire and in what sequence. Plan the order carefully when combining multiple switches.
