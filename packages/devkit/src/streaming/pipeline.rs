use async_trait::async_trait;
use chrono::Utc;
use reqwest::Client;
use std::collections::VecDeque;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::error::DevkitError;
use crate::protocol::fee_stats::HorizonFeeStats;
use crate::storage::{traits::FeeRecord as StoredFeeRecord, FeeStore};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamRecord {
    pub timestamp_ms: u64,
    pub fee_stroops: u64,
    pub sequence: u64,
}

#[async_trait]
pub trait Source: Send {
    async fn next(&mut self) -> Result<Option<StreamRecord>, DevkitError>;
}

pub trait Transformer: Send {
    fn transform(&mut self, record: StreamRecord) -> Option<StreamRecord>;
}

pub trait Transform: Transformer {}

#[async_trait]
pub trait Sink: Send {
    async fn write(&mut self, record: StreamRecord) -> Result<(), DevkitError>;
}

pub struct Pipeline {
    source: Box<dyn Source>,
    transforms: Vec<Box<dyn Transformer>>,
    sink: Box<dyn Sink>,
    capacity: usize,
}

pub struct PipelineBuilder {
    source: Option<Box<dyn Source>>,
    transforms: Vec<Box<dyn Transformer>>,
    sink: Option<Box<dyn Sink>>,
    capacity: usize,
}

impl Pipeline {
    pub fn builder() -> PipelineBuilder {
        PipelineBuilder {
            source: None,
            transforms: Vec::new(),
            sink: None,
            capacity: 64,
        }
    }

    pub async fn run(mut self) -> Result<usize, DevkitError> {
        let (sender, mut receiver) = mpsc::channel(self.capacity);
        let mut source = self.source;
        let producer = tokio::spawn(async move {
            while let Some(record) = source.next().await? {
                sender.send(record).await.map_err(|_| {
                    DevkitError::Storage("pipeline sink stopped consuming events".to_string())
                })?;
            }
            Ok::<(), DevkitError>(())
        });

        let mut written = 0;
        while let Some(mut record) = receiver.recv().await {
            let mut keep = true;
            for transform in &mut self.transforms {
                match transform.transform(record) {
                    Some(transformed) => record = transformed,
                    None => {
                        keep = false;
                        break;
                    }
                }
            }
            if keep {
                self.sink.write(record).await?;
                written += 1;
            }
        }
        producer.await.map_err(|error| {
            DevkitError::Storage(format!("pipeline source task failed: {error}"))
        })??;
        Ok(written)
    }
}

impl PipelineBuilder {
    pub fn source<T: Source + 'static>(mut self, source: T) -> Self {
        self.source = Some(Box::new(source));
        self
    }

    pub fn transform<T: Transformer + 'static>(mut self, transform: T) -> Self {
        self.transforms.push(Box::new(transform));
        self
    }

    pub fn sink<T: Sink + 'static>(mut self, sink: T) -> Self {
        self.sink = Some(Box::new(sink));
        self
    }

    pub fn capacity(mut self, capacity: usize) -> Self {
        self.capacity = capacity.max(1);
        self
    }

    pub fn build(self) -> Result<Pipeline, DevkitError> {
        Ok(Pipeline {
            source: self
                .source
                .ok_or_else(|| DevkitError::Storage("pipeline source is required".to_string()))?,
            transforms: self.transforms,
            sink: self
                .sink
                .ok_or_else(|| DevkitError::Storage("pipeline sink is required".to_string()))?,
            capacity: self.capacity,
        })
    }
}

#[derive(Debug, Clone)]
pub struct PollingConfig {
    pub endpoint: String,
    pub interval: std::time::Duration,
    pub max_polls: Option<usize>,
}

pub struct PollingSource {
    client: Client,
    config: PollingConfig,
    polls: usize,
    sequence: u64,
}

impl PollingSource {
    pub fn new(config: PollingConfig) -> Self {
        Self {
            client: Client::new(),
            config,
            polls: 0,
            sequence: 0,
        }
    }
}

#[async_trait]
impl Source for PollingSource {
    async fn next(&mut self) -> Result<Option<StreamRecord>, DevkitError> {
        if self.config.max_polls.is_some_and(|limit| self.polls >= limit) {
            return Ok(None);
        }
        if self.polls > 0 {
            tokio::time::sleep(self.config.interval).await;
        }
        let response = self
            .client
            .get(&self.config.endpoint)
            .send()
            .await
            .map_err(|error| DevkitError::Protocol(error.to_string()))?
            .error_for_status()
            .map_err(|error| DevkitError::Protocol(error.to_string()))?
            .json::<HorizonFeeStats>()
            .await
            .map_err(|error| DevkitError::Protocol(error.to_string()))?;
        self.polls += 1;
        self.sequence += 1;
        Ok(Some(StreamRecord {
            timestamp_ms: Utc::now().timestamp_millis() as u64,
            fee_stroops: response.last_ledger_base_fee,
            sequence: self.sequence,
        }))
    }
}

pub struct FileReplaySource {
    records: VecDeque<StreamRecord>,
}

impl FileReplaySource {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, DevkitError> {
        let file = File::open(path).map_err(DevkitError::Io)?;
        let mut lines = BufReader::new(file).lines();
        let header = lines.next().transpose().map_err(DevkitError::Io)?;
        if header.as_deref() != Some("timestamp_ms,fee_stroops,sequence") {
            return Err(invalid_csv("expected timestamp_ms,fee_stroops,sequence CSV header"));
        }
        let records = lines
            .map(|line| line.map_err(DevkitError::Io))
            .map(|line| {
                let fields: Vec<_> = line?.split(',').map(str::to_string).collect();
                if fields.len() != 3 {
                    return Err(invalid_csv("expected three CSV fields"));
                }
                Ok(StreamRecord {
                    timestamp_ms: fields[0].parse().map_err(invalid_csv)?,
                    fee_stroops: fields[1].parse().map_err(invalid_csv)?,
                    sequence: fields[2].parse().map_err(invalid_csv)?,
                })
            })
            .collect::<Result<VecDeque<_>, _>>()?;
        Ok(Self { records })
    }
}

fn invalid_csv(error: impl std::fmt::Display) -> DevkitError {
    DevkitError::Io(io::Error::new(io::ErrorKind::InvalidData, error.to_string()))
}

#[async_trait]
impl Source for FileReplaySource {
    async fn next(&mut self) -> Result<Option<StreamRecord>, DevkitError> {
        Ok(self.records.pop_front())
    }
}

pub struct SpikeDetector {
    threshold: u64,
}

impl SpikeDetector {
    pub fn new(threshold: u64) -> Self {
        Self { threshold }
    }
}

impl Transformer for SpikeDetector {
    fn transform(&mut self, record: StreamRecord) -> Option<StreamRecord> {
        (record.fee_stroops >= self.threshold).then_some(record)
    }
}

pub struct RollingAverageTransformer {
    window: usize,
    values: VecDeque<u64>,
}

impl RollingAverageTransformer {
    pub fn new(window: usize) -> Self {
        Self {
            window: window.max(1),
            values: VecDeque::new(),
        }
    }
}

impl Transformer for RollingAverageTransformer {
    fn transform(&mut self, mut record: StreamRecord) -> Option<StreamRecord> {
        self.values.push_back(record.fee_stroops);
        if self.values.len() > self.window {
            self.values.pop_front();
        }
        record.fee_stroops = self.values.iter().sum::<u64>() / self.values.len() as u64;
        Some(record)
    }
}

pub struct StorageSink {
    store: Arc<dyn FeeStore>,
}

impl StorageSink {
    pub fn new(store: Arc<dyn FeeStore>) -> Self {
        Self { store }
    }
}

#[async_trait]
impl Sink for StorageSink {
    async fn write(&mut self, record: StreamRecord) -> Result<(), DevkitError> {
        self.store
            .insert(StoredFeeRecord {
                fee_amount: record.fee_stroops,
                ledger_sequence: record.sequence,
                timestamp_ms: record.timestamp_ms as i64,
                transaction_hash: None,
                is_spike: false,
                created_at: Utc::now().to_rfc3339(),
            })
            .await
    }
}