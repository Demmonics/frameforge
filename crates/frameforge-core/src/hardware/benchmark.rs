use serde::{Deserialize, Serialize};

/// Benchmark descriptor for Milestone F hardware vs software throughput measurement.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EncoderBenchmarkResult {
    pub encoder_name: String,
    pub fps: f64,
    pub elapsed_ms: u64,
    pub output_bytes: u64,
}
