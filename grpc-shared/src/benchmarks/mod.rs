//! Performance benchmarking module
//!
//! This module provides benchmarking infrastructure as specified in PRD Phase 7: Benchmarking & Testing

#[cfg(feature = "benchmarks")]
pub mod rpc_benchmark;

#[cfg(feature = "benchmarks")]
pub use rpc_benchmark::BenchmarkRunner;

/// Benchmark configuration
#[derive(Debug, Clone)]
pub struct BenchmarkConfig {
    /// Target requests per second
    pub target_rps: u32,
    /// Duration in seconds
    pub duration_seconds: u64,
    /// Number of connections
    pub num_connections: u32,
    /// Number of threads
    pub num_threads: u32,
    /// Key type for testing
    pub key_type: crate::config::KeyType,
    /// Transport type for testing
    pub transport: crate::config::TransportType,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            target_rps: 1000,
            duration_seconds: 60,
            num_connections: 10,
            num_threads: 4,
            key_type: crate::config::KeyType::EccP256,
            transport: crate::config::TransportType::Tcp,
        }
    }
}