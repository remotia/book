# Pipelines & lifecycle

A pipeline is a directed chain of components that process frames sequentially. Each component runs concurrently as its own async task, while processors within a component execute in order. Pipelines can be connected via switches — special processors that route DTOs from one pipeline to another — enabling complex architectures such as a main streaming pipeline with a separate error-handling pipeline and a profiling side-channel. The framework manages channel wiring and task spawning automatically: calling `.run()` creates unbounded `mpsc` channels between adjacent components, spawns each as a Tokio task, and returns join handles for lifecycle management. See the [pipeline module documentation](https://docs.rs/remotia/latest/remotia/pipeline/index.html) for the full API reference.

This page covers the full `Pipeline`, `PipelineFeeder`, `PipelineRegistry`, and `PipelineHandle` APIs, plus the shutdown/drain lifecycle.

---

## Pipeline

A `Pipeline<F>` is an ordered chain of `Component<F>` instances. When you call `.run()`, the framework:

1. **Binds** — creates unbounded `mpsc` channels between each pair of adjacent components.
2. **Spawns** — launches each component as a separate Tokio task.
3. Returns `Vec<JoinHandle<()>>` you can await.

### Builder API

```rust
use remotia::pipeline::Pipeline;

let handles = Pipeline::new()
    .link(capture_component)
    .link(encoder_component)
    .link(transmission_component)
    .tag("main")
    .run();
```

| Method | Description |
|---|---|
| `Pipeline::new()` | Create an empty pipeline |
| `Pipeline::singleton(component)` | Create a one-component pipeline |
| `.link(component)` | Append a component |
| `.tag(name)` | Set a tag for log messages |
| `.feedable()` | Mark the pipeline to accept external DTOs (call before `.run()`) |
| `.get_feeder()` | Get a `PipelineFeeder` for injecting DTOs (pipeline must be feedable) |
| `.get_handle()` | Get a `PipelineHandle` for requesting shutdown |
| `.shutdown_signal()` | Get the `Arc<AtomicBool>` shutdown signal |
| `.run()` | Bind channels, spawn tasks, return join handles |

---

## PipelineFeeder

A `PipelineFeeder<F>` lets you inject DTOs into the head of a feedable pipeline from external code.

```rust
let mut pipeline = Pipeline::new()
    .link(component)
    .feedable();

let feeder = pipeline.get_feeder();

// Later, from any task:
feeder.feed(my_dto); // panics if the channel is closed
```

`PipelineFeeder` is `Clone`-safe — you can share it across tasks.

---

## PipelineHandle

A `PipelineHandle` lets you request graceful shutdown of a pipeline.

```rust
let mut pipeline = Pipeline::new().link(component).tag("main");
let handle = pipeline.get_handle();
let handles = pipeline.run();

// From another task:
handle.request_shutdown();
```

When `request_shutdown()` is called, the shared `AtomicBool` is set. Every component in the pipeline checks this signal each iteration and exits when it becomes `true`.

`PipelineHandle` is `Clone` — multiple callers can hold a handle to the same pipeline. See the [pipeline module documentation](https://docs.rs/remotia/latest/remotia/pipeline/index.html) for the full PipelineHandle API reference.

---

## PipelineRegistry

`PipelineRegistry<F, K>` manages multiple named pipelines and runs them together. This is the standard way to launch a multi-pipeline architecture.

### Builder API

```rust
use remotia::pipeline::registry::PipelineRegistry;

let mut registry = PipelineRegistry::<MyDto, &str>::new();

registry.register("main", main_pipeline);
registry.register("errors", error_pipeline);

registry.run().await; // runs all pipelines, blocks until all finish
```

| Method | Description |
|---|---|
| `PipelineRegistry::new()` | Create an empty registry |
| `.register(id, pipeline)` | Insert a pipeline with a key |
| `.register_empty(id)` | Insert an empty pipeline |
| `.get(&id)` / `.get_mut(&mut id)` | Access a pipeline by key |
| `.lazy_handle(id)` | Get a `PipelineHandle` for a pipeline (works even before `.run()`) |
| `.run()` | Bind and spawn all pipelines, await all tasks |

See the [pipeline registry module documentation](https://docs.rs/remotia/latest/remotia/pipeline/registry/index.html) for the full PipelineRegistry API reference.

### Connecting pipelines with switches

When using a registry, you typically get feeders from destination pipelines *before* calling `.run()`:

```rust
let mut registry = PipelineRegistry::<MyDto, &str>::new();

let error_pipeline = Pipeline::new()
    .link(error_component)
    .tag("errors");

registry.register("errors", error_pipeline);

let error_switch = {
    let error_pipe = registry.get_mut(&"errors");
    OnErrorSwitch::new(error_pipe).detect(MyError::Timeout)
};

let main_pipeline = Pipeline::new()
    .link(capture_component.append(ticker).append(error_switch))
    .tag("main");

registry.register("main", main_pipeline);
registry.run().await;
```

---

## Lifecycle: shutdown and drain

Understanding the component lifecycle is important for graceful shutdown.

### Normal operation

```
Component receives DTO from input channel
  → passes through processors
  → sends result to output channel
  → repeats
```

### Shutdown signal

When `PipelineHandle::request_shutdown()` is called, the shared `AtomicBool` is set. On the next iteration, each component observes the signal and breaks out of its loop.

### Drain mode

If a component's **input channel** is closed (e.g. the upstream component has exited), the component enters **drain mode**:

1. It stops reading from the channel (it will always be `None`).
2. It continues the loop, yielding `F::default()` as the DTO, and running processors.
3. When a processor returns `None` (frame consumed), the component checks if it should exit.
4. If the input channel was closed *and* the last processor returned `None`, the component exits.

This ensures that in-flight DTOs in downstream components can finish processing before shutdown.

### Component without an input channel

A component with no receiver (e.g. the head of a non-feedable pipeline that generates its own DTOs) relies solely on the shutdown signal. It checks `is_shutdown()` each iteration and exits when the signal fires.

---

## Channel binding details

When `.run()` is called on a pipeline that has not been bound, the framework automatically creates an unbounded `mpsc` channel between each pair of adjacent components:

```
Component[i]  ──sender──▶  channel  ──receiver──▶  Component[i+1]
```

The last component has no sender; the first component has no receiver unless the pipeline is feedable (in which case the feeder's sender is wired to the first component's receiver).
