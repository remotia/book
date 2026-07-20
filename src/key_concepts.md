# Key concepts

Remotia is built around four abstractions that compose into streaming pipelines. This page defines each one and shows the Rust APIs you implement and use.

```
Pipeline
 ┌──────────────────────────┐       ┌──────────────────────────┐
 │ Component A               │       │ Component B               │
 │  ┌──────────┐ ┌─────────┐│  ch   │  ┌──────────┐ ┌─────────┐│
 │  │Processor 1│→│Processor 2││──────▶│  │Processor 3│→│Processor 4││
 │  └──────────┘ └─────────┘│       │  └──────────┘ └─────────┘│
 └──────────────────────────┘       └──────────────────────────┘
```

- A **DTO** (Data Transfer Object) carries frame data and metadata through the pipeline.
- A **processor** performs one atomic operation on a DTO.
- A **component** groups processors into a single async task, connected to neighbors by channels.
- A **pipeline** links components into a directed chain; multiple pipelines are connected by **switches**.

---

## Data Transfer Object (DTO)

<img style="display:block; margin: auto" src="./figures/frame_dto.svg">

The DTO is the data structure that flows through the pipeline. It typically holds frame buffers and per-frame statistics. You define your own DTO type and implement the traits that the processors in your pipeline require.

### Core trait: `FrameProcessor` input

Every processor operates on a generic type `F`. Your DTO is that type `F`. The framework does not mandate a specific struct — you define one adapted to your use case.

### Trait: `FrameProperties<K, V>`

Used by switches that read or write routing keys on the DTO (e.g. `PoolingSwitch`, `DepoolingSwitch`).

```rust
pub trait FrameProperties<K, V> {
    fn set(&mut self, key: K, value: V);
    fn get(&self, key: &K) -> Option<V>;
}
```

### Trait: `FrameError<E>`

Used by `OnErrorSwitch` to inspect error state on the DTO.

```rust
pub trait FrameError<E> {
    fn report_error(&mut self, error: E);
    fn get_error(&self) -> Option<E>;
}
```

### Other available traits

| Trait | Purpose | Used by |
|---|---|---|
| `PullableFrameProperties<K, V>` | Push/pull semantics for properties | Advanced routing |
| `OptionalFrameData<D>` | Access optional embedded data | Buffer utilities |
| `BorrowFrameProperties<K, V>` | Get a reference to a property value | Read-only inspection |
| `BorrowMutFrameProperties<K, V>` | Get a mutable reference to a property value | In-place mutation |

### Minimal DTO example

```rust
use remotia::traits::{FrameProperties, FrameError};

#[derive(Debug, Default)]
struct MyDto {
    buffer: Vec<u8>,
    frame_id: u64,
    error: Option<String>,
}

impl FrameProperties<String, u64> for MyDto {
    fn set(&mut self, key: String, value: u64) {
        if key == "frame_id" { self.frame_id = value; }
    }
    fn get(&self, key: &String) -> Option<u64> {
        if key == "frame_id" { Some(self.frame_id) } else { None }
    }
}

impl FrameError<String> for MyDto {
    fn report_error(&mut self, error: String) { self.error = Some(error); }
    fn get_error(&self) -> Option<String> { self.error.clone() }
}
```

Custom processor modules may define additional traits. Your DTO must implement them if you use those modules in your pipeline.

---

## Processors

<img style="display:block; margin: auto; width: 30em" src="./figures/processor.svg">

A processor is a single unit of work applied to a DTO. The core trait is:

```rust
#[async_trait]
pub trait FrameProcessor<F> {
    async fn process(&mut self, frame_data: F) -> Option<F>;
}
```

**Return contract:**
- `Some(dto)` — the DTO is passed to the next processor in the component.
- `None` — the DTO is consumed. The pipeline interprets this as "this frame is done here" — it may have been redirected to another pipeline (by a switch), stored, or dropped.

Processors have full [ownership](https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html) of the DTO while processing it. This avoids borrowing conflicts and makes the data flow explicit.

See the [Processors](./processors.md) page for the catalog of built-in processor types.

---

## Components

<img style="display:block; margin: auto" src="./figures/component.svg">

A **component** groups an ordered sequence of processors into a single async task (a [Tokio task](https://tokio.rs/tokio/tutorial/spawning#tasks)). Each component:

1. Receives a DTO from an input channel (or allocates one itself).
2. Passes it through its processors sequentially.
3. Sends the resulting DTO (if any) to the next component via an output channel.

Components are the unit of concurrency. By grouping processors into different components, you control which work shares a task and which runs in parallel. The framework uses unbounded `mpsc` channels to connect adjacent components within a pipeline.

### Builder API

```rust
Component::new()
    .append(processor_a)
    .append(processor_b)
    .tag("encoder")
```

Or, for a single-processor component:

```rust
Component::singleton(processor)
```

---

## Pipelines

<img style="display:block; margin: auto" src="./figures/pipeline.svg">

A **pipeline** is a chain of components connected by channels. Components within a pipeline run concurrently (each is a separate Tokio task), while processors within a component run sequentially.

### Builder API

```rust
Pipeline::new()
    .link(component_a)
    .link(component_b)
    .tag("main")
    .run()
```

Or, for a single-component pipeline:

```rust
Pipeline::singleton(component)
```

Calling `.run()` automatically creates the channels between adjacent components and spawns each component as a Tokio task. It returns `Vec<JoinHandle<()>>`.

### Feedable pipelines

Mark a pipeline as `.feedable()` to allow external code to inject DTOs into its head:

```rust
let mut pipeline = Pipeline::new().link(component).feedable();
let feeder = pipeline.get_feeder();
feeder.feed(my_dto);
```

### Multi-pipeline architectures

Complex systems use multiple pipelines connected by switches. For example, a main streaming pipeline and a separate error-handling pipeline, linked by an `OnErrorSwitch`. See the [Pipelines & Lifecycle](./pipeline-lifecycle.md) page for the full API and lifecycle details.

---

## Switches

**Switches** are processors that move DTOs between pipelines. Instead of returning `Some(dto)` to continue in the current pipeline, they send the DTO to a different pipeline and return `None`.

The framework provides several switch types:

| Switch | Behavior |
|---|---|
| `Switch` | Unconditionally redirects the DTO to another pipeline |
| `CloneSwitch` | Clones the DTO, sends the clone to another pipeline, passes the original forward |
| `OnErrorSwitch` | Redirects the DTO to another pipeline if it carries a matching error |
| `PoolingSwitch` | Picks a random destination from a pool and stamps the DTO with the pool key |
| `DepoolingSwitch` | Routes the DTO to the destination matching its pool key |

See the [Processors](./processors.md) page for constructor signatures and usage details.
