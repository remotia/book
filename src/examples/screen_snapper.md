# Screen snapper — Capture the screen and save to disk

Periodically captures the primary display and saves each frame as a PNG file.

Full source: [GitHub — screen-snapper](https://github.com/remotia/examples/tree/main/screen-snapper)

## Architecture

Two components in a single pipeline:

```
Component 1: Capture                          Component 2: Save
┌──────────────────────────────────┐          ┌─────────────────┐
│ Ticker → BufferAllocator →       │  channel │ PNGBufferSaver  │
│ XCapCapturer                     │─────────▶│                 │
└──────────────────────────────────┘          └─────────────────┘
```

- **Component 1** paces the capture at 1 FPS, allocates a buffer, fills it with screen pixels.
- **Component 2** reads the buffer and writes a PNG.

## The DTO

The DTO holds a single optional `BytesMut` buffer identified by an enum key. It implements `PullableFrameProperties` so processors can pull and push the buffer by key.

```rust
#[derive(Default, Debug)]
pub struct RecorderData {
    screen_buffer: Option<BytesMut>,
}

#[derive(Clone, Copy)]
pub enum Buffers {
    CapturedScreenBuffer,
}

impl PullableFrameProperties<Buffers, BytesMut> for RecorderData {
    fn push(&mut self, key: Buffers, value: BytesMut) {
        match key {
            Buffers::CapturedScreenBuffer => self.screen_buffer.replace(value),
        };
    }
    fn pull(&mut self, key: &Buffers) -> Option<BytesMut> {
        match key {
            Buffers::CapturedScreenBuffer => self.screen_buffer.take(),
        }
    }
}
```

The `pull`/`push` pattern ensures the buffer is taken out of the DTO during processing and returned after, avoiding mutable aliasing.

## Pipeline construction

```rust
fn capturer(monitor_id: usize, height: u32, width: u32) -> Component<RecorderData> {
    Component::new()
        .append(Ticker::new(1000))       // 1 tick/second
        .append(BufferAllocator::new(    // allocate a fresh BytesMut
            Buffers::CapturedScreenBuffer,
            height as usize * width as usize * 3,
        ))
        .append(
            XCapCapturer::builder()
                .buffer_key(Buffers::CapturedScreenBuffer)
                .monitor_id(monitor_id)
                .build(),
        )
}

fn saver(height: u32, width: u32) -> Component<RecorderData> {
    Component::new().append(
        PNGBufferSaver::builder()
            .buffer_key(Buffers::CapturedScreenBuffer)
            .path("./screenshots/")
            .height(height)
            .width(width)
            .build(),
    )
}

let pipeline = Pipeline::<RecorderData>::new()
    .link(capturer(monitor_id, height, width))
    .link(saver(height, width));

for handle in pipeline.run() {
    handle.await.unwrap();
}
```

## Key takeaways

- `Ticker` at the head of a component paces frame generation.
- `BufferAllocator` is a processor that allocates a new `BytesMut` into the DTO each tick — useful when the buffer size is known ahead of time.
- Custom processors (`XCapCapturer`, `PNGBufferSaver`) implement `FrameProcessor` and use `PullableFrameProperties` to exchange buffers with the DTO.
