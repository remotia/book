# Crate map

Remotia is organized as a workspace of crates. The `remotia` umbrella crate re-exports them behind feature flags so you only compile what you need.

## Core crate

| Feature flag | Crate | Purpose | Key types |
|---|---|---|---|
| *(always enabled)* | `remotia-core` | Core traits, pipeline engine, built-in processors | `FrameProcessor`, `FrameProperties`, `FrameError`, `Pipeline`, `Component`, `PipelineFeeder`, `PipelineHandle`, `PipelineRegistry`, `Ticker`, `Switch`, `CloneSwitch`, `OnErrorSwitch`, `PoolingSwitch`, `DepoolingSwitch`, `Sequential`, `Function`, `Closure`, `AsyncFunction` |

## Optional crates (feature flags on the `remotia` crate)

| Feature flag | Crate(s) | Purpose | Key types |
|---|---|---|---|
| `buffers` | `remotia-buffer-utils`, `remotia-buffer-utils-macros` | Buffer pool management and allocation | `BuffersPool`, `BufferAllocator`, `BufferBorrower`, `BufferRedeemer`, `PoolRegistry`, `#[buffers_map]` macro |
| `capture` | `remotia-core-capturers` | Screen and file capture | `ScrapFrameCapturer`, `Y4MFrameCapturer`, `Y4MRGBAFrameCapturer`, `yuv420_to_rgba()` |
| `render` | `remotia-core-renderers` | Window rendering via winit + pixels | `WinitRenderer`, `WinitRunner` |
| `transmission` | `remotia-core-transmission` | TCP frame transport | `TcpFrameSender`, `TcpFrameReceiver` |
| `profilation` | `remotia-profilation-utils` | Profiling, logging, frame-dropping | `TimestampAdder`, `TimestampDiffCalculator`, `ProfiledSequential`, `ThresholdBasedFrameDropper`, `TimestampBasedFrameDropper`, `ConsoleAverageStatsLogger`, `CSVFrameDataSerializer`, `ConsoleDropReasonLogger` |
| `serialization` | `remotia-serialization-utils` | Bincode serialization of frame data | `BincodeSerializer`, `BincodeDeserializer` |

## External crates (separate Cargo dependencies)

These live in separate repositories and are not part of the `remotia` umbrella crate. Add them as direct dependencies.

| Crate | Purpose | Key types |
|---|---|---|
| [`remotia-ffmpeg-codecs`](https://github.com/remotia/remotia-ffmpeg-codecs) | FFmpeg encoder/decoder integration via pusher/puller architecture | `EncoderPusher`, `EncoderPuller`, `DecoderPusher`, `DecoderPuller`, `EncoderBuilder`, `DecoderBuilder`, `ScalerBuilder` |
| [`remotia-srt`](https://github.com/remotia/remotia-srt) | SRT protocol sender/receiver processors | SRT sender/receiver processors |

## Feature flag usage

```toml
[dependencies.remotia]
version = "0.1.0"
default-features = false
features = ["capture", "transmission"]  # only what you need
```

All features are off by default. Enable only the ones you use to minimize compile time and dependencies.
