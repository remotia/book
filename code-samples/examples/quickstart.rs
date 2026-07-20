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
