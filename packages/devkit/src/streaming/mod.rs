//! Composable, bounded streaming primitives for fee observations.

mod pipeline;

pub use pipeline::{
    FileReplaySource, Pipeline, PipelineBuilder, PollingConfig, PollingSource, Sink, Source,
    SpikeDetector, Transform, Transformer, RollingAverageTransformer, StorageSink, StreamRecord,
};