//! Benchmark tool for testing gRPC performance optimizations

use grpc_performance_rs::{
    echo::{echo_service_client::EchoServiceClient, EchoRequest},
    crypto::{
        crypto_service_client::CryptoServiceClient,
        SignRequest, KeyType, SigningAlgorithm
    },
    current_timestamp_millis, AppResult, DEFAULT_SERVER_ADDR,
};
use log::info;
use std::env;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use tonic::transport::Channel;

#[derive(Clone)]
struct BenchmarkMetrics {
    total_requests: Arc<AtomicU64>,
    successful_requests: Arc<AtomicU64>,
    failed_requests: Arc<AtomicU64>,
    total_latency_micros: Arc<AtomicU64>,
    min_latency_micros: Arc<AtomicU64>,
    max_latency_micros: Arc<AtomicU64>,
}

impl BenchmarkMetrics {
    fn new() -> Self {
        Self {
            total_requests: Arc::new(AtomicU64::new(0)),
            successful_requests: Arc::new(AtomicU64::new(0)),
            failed_requests: Arc::new(AtomicU64::new(0)),
            total_latency_micros: Arc::new(AtomicU64::new(0)),
            min_latency_micros: Arc::new(AtomicU64::new(u64::MAX)),
            max_latency_micros: Arc::new(AtomicU64::new(0)),
        }
    }

    fn record_success(&self, latency_micros: u64) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        self.successful_requests.fetch_add(1, Ordering::Relaxed);
        self.total_latency_micros.fetch_add(latency_micros, Ordering::Relaxed);
        
        // Update min latency
        let mut current_min = self.min_latency_micros.load(Ordering::Relaxed);
        while latency_micros < current_min {
            match self.min_latency_micros.compare_exchange_weak(
                current_min, latency_micros, Ordering::Relaxed, Ordering::Relaxed
            ) {
                Ok(_) => break,
                Err(x) => current_min = x,
            }
        }
        
        // Update max latency
        let mut current_max = self.max_latency_micros.load(Ordering::Relaxed);
        while latency_micros > current_max {
            match self.max_latency_micros.compare_exchange_weak(
                current_max, latency_micros, Ordering::Relaxed, Ordering::Relaxed
            ) {
                Ok(_) => break,
                Err(x) => current_max = x,
            }
        }
    }

    fn record_failure(&self) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        self.failed_requests.fetch_add(1, Ordering::Relaxed);
    }

    fn get_stats(&self) -> (u64, u64, u64, f64, u64, u64) {
        let total = self.total_requests.load(Ordering::Relaxed);
        let successful = self.successful_requests.load(Ordering::Relaxed);
        let failed = self.failed_requests.load(Ordering::Relaxed);
        let total_latency = self.total_latency_micros.load(Ordering::Relaxed);
        let min_latency = self.min_latency_micros.load(Ordering::Relaxed);
        let max_latency = self.max_latency_micros.load(Ordering::Relaxed);
        
        let avg_latency = if successful > 0 { 
            total_latency as f64 / successful as f64 
        } else { 
            0.0 
        };
        
        (total, successful, failed, avg_latency, min_latency, max_latency)
    }
}

async fn create_optimized_channel(addr: &str) -> AppResult<Channel> {
    let channel = Channel::from_shared(format!("http://{}", addr))?
        .tcp_keepalive(Some(Duration::from_secs(30)))
        .tcp_nodelay(true)
        .http2_keep_alive_interval(Duration::from_secs(30))
        .keep_alive_timeout(Duration::from_secs(5))
        .initial_stream_window_size(Some(1024 * 1024)) // 1MB
        .initial_connection_window_size(Some(1024 * 1024)) // 1MB
        .connect()
        .await?;
    
    Ok(channel)
}

async fn benchmark_echo_service(
    addr: &str,
    concurrent_requests: usize,
    total_requests: usize,
    metrics: BenchmarkMetrics,
) -> AppResult<()> {
    info!("Starting echo service benchmark: {} concurrent, {} total requests", 
          concurrent_requests, total_requests);
    
    let semaphore = Arc::new(Semaphore::new(concurrent_requests));
    let mut tasks = Vec::new();
    
    for i in 0..total_requests {
        let semaphore = semaphore.clone();
        let metrics = metrics.clone();
        let addr = addr.to_string();
        
        let task = tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            
            let start_time = Instant::now();
            
            match create_optimized_channel(&addr).await {
                Ok(channel) => {
                    let mut client = EchoServiceClient::new(channel);
                    let request = EchoRequest {
                        payload: format!("Benchmark request {}", i),
                        timestamp: current_timestamp_millis(),
                    };
                    
                    match client.echo(request).await {
                        Ok(_) => {
                            let latency = start_time.elapsed().as_micros() as u64;
                            metrics.record_success(latency);
                        }
                        Err(_) => {
                            metrics.record_failure();
                        }
                    }
                }
                Err(_) => {
                    metrics.record_failure();
                }
            }
        });
        
        tasks.push(task);
    }
    
    // Wait for all tasks to complete
    for task in tasks {
        let _ = task.await;
    }
    
    Ok(())
}

async fn benchmark_crypto_service(
    addr: &str,
    concurrent_requests: usize,
    total_requests: usize,
    metrics: BenchmarkMetrics,
) -> AppResult<()> {
    info!("Starting crypto service benchmark: {} concurrent, {} total requests", 
          concurrent_requests, total_requests);
    
    let semaphore = Arc::new(Semaphore::new(concurrent_requests));
    let mut tasks = Vec::new();
    
    for i in 0..total_requests {
        let semaphore = semaphore.clone();
        let metrics = metrics.clone();
        let addr = addr.to_string();
        
        let task = tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            
            let start_time = Instant::now();
            
            match create_optimized_channel(&addr).await {
                Ok(channel) => {
                    let mut client = CryptoServiceClient::new(channel);
                    let request = SignRequest {
                        data: format!("Benchmark data {}", i).into_bytes(),
                        key_type: KeyType::Rsa as i32,
                        algorithm: SigningAlgorithm::RsaPkcs1Sha256 as i32,
                        timestamp: current_timestamp_millis(),
                    };
                    
                    match client.sign(request).await {
                        Ok(_) => {
                            let latency = start_time.elapsed().as_micros() as u64;
                            metrics.record_success(latency);
                        }
                        Err(_) => {
                            metrics.record_failure();
                        }
                    }
                }
                Err(_) => {
                    metrics.record_failure();
                }
            }
        });
        
        tasks.push(task);
    }
    
    // Wait for all tasks to complete
    for task in tasks {
        let _ = task.await;
    }
    
    Ok(())
}

#[tokio::main]
async fn main() -> AppResult<()> {
    // Initialize logging
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    // Parse configuration from environment variables
    let server_addr = env::var("SERVER_ADDR")
        .unwrap_or_else(|_| DEFAULT_SERVER_ADDR.to_string());
    
    let concurrent_requests: usize = env::var("CONCURRENT_REQUESTS")
        .unwrap_or_else(|_| "10".to_string())
        .parse()
        .unwrap_or(10);
    
    let total_requests: usize = env::var("TOTAL_REQUESTS")
        .unwrap_or_else(|_| "1000".to_string())
        .parse()
        .unwrap_or(1000);
    
    let benchmark_type = env::var("BENCHMARK_TYPE")
        .unwrap_or_else(|_| "both".to_string());

    info!("Starting gRPC performance benchmark");
    info!("Server: {}", server_addr);
    info!("Concurrent requests: {}", concurrent_requests);
    info!("Total requests: {}", total_requests);
    info!("Benchmark type: {}", benchmark_type);

    let overall_start = Instant::now();

    match benchmark_type.as_str() {
        "echo" => {
            let metrics = BenchmarkMetrics::new();
            let start_time = Instant::now();
            
            benchmark_echo_service(&server_addr, concurrent_requests, total_requests, metrics.clone()).await?;
            
            let duration = start_time.elapsed();
            let (total, successful, failed, avg_latency, min_latency, max_latency) = metrics.get_stats();
            
            info!("\n=== Echo Service Benchmark Results ===");
            info!("Total requests: {}", total);
            info!("Successful requests: {}", successful);
            info!("Failed requests: {}", failed);
            info!("Success rate: {:.2}%", (successful as f64 / total as f64) * 100.0);
            info!("Duration: {:?}", duration);
            info!("Requests per second: {:.2}", successful as f64 / duration.as_secs_f64());
            info!("Average latency: {:.2} μs", avg_latency);
            info!("Min latency: {} μs", if min_latency == u64::MAX { 0 } else { min_latency });
            info!("Max latency: {} μs", max_latency);
        }
        "crypto" => {
            let metrics = BenchmarkMetrics::new();
            let start_time = Instant::now();
            
            benchmark_crypto_service(&server_addr, concurrent_requests, total_requests, metrics.clone()).await?;
            
            let duration = start_time.elapsed();
            let (total, successful, failed, avg_latency, min_latency, max_latency) = metrics.get_stats();
            
            info!("\n=== Crypto Service Benchmark Results ===");
            info!("Total requests: {}", total);
            info!("Successful requests: {}", successful);
            info!("Failed requests: {}", failed);
            info!("Success rate: {:.2}%", (successful as f64 / total as f64) * 100.0);
            info!("Duration: {:?}", duration);
            info!("Requests per second: {:.2}", successful as f64 / duration.as_secs_f64());
            info!("Average latency: {:.2} μs", avg_latency);
            info!("Min latency: {} μs", if min_latency == u64::MAX { 0 } else { min_latency });
            info!("Max latency: {} μs", max_latency);
        }
        "both" | _ => {
            // Benchmark echo service
            let echo_metrics = BenchmarkMetrics::new();
            let echo_start = Instant::now();
            
            benchmark_echo_service(&server_addr, concurrent_requests, total_requests, echo_metrics.clone()).await?;
            
            let echo_duration = echo_start.elapsed();
            let (echo_total, echo_successful, echo_failed, echo_avg_latency, echo_min_latency, echo_max_latency) = echo_metrics.get_stats();
            
            // Benchmark crypto service
            let crypto_metrics = BenchmarkMetrics::new();
            let crypto_start = Instant::now();
            
            benchmark_crypto_service(&server_addr, concurrent_requests, total_requests, crypto_metrics.clone()).await?;
            
            let crypto_duration = crypto_start.elapsed();
            let (crypto_total, crypto_successful, crypto_failed, crypto_avg_latency, crypto_min_latency, crypto_max_latency) = crypto_metrics.get_stats();
            
            info!("\n=== Combined Benchmark Results ===");
            info!("Echo Service:");
            info!("  Total requests: {}", echo_total);
            info!("  Successful requests: {}", echo_successful);
            info!("  Failed requests: {}", echo_failed);
            info!("  Success rate: {:.2}%", (echo_successful as f64 / echo_total as f64) * 100.0);
            info!("  Duration: {:?}", echo_duration);
            info!("  Requests per second: {:.2}", echo_successful as f64 / echo_duration.as_secs_f64());
            info!("  Average latency: {:.2} μs", echo_avg_latency);
            info!("  Min latency: {} μs", if echo_min_latency == u64::MAX { 0 } else { echo_min_latency });
            info!("  Max latency: {} μs", echo_max_latency);
            
            info!("Crypto Service:");
            info!("  Total requests: {}", crypto_total);
            info!("  Successful requests: {}", crypto_successful);
            info!("  Failed requests: {}", crypto_failed);
            info!("  Success rate: {:.2}%", (crypto_successful as f64 / crypto_total as f64) * 100.0);
            info!("  Duration: {:?}", crypto_duration);
            info!("  Requests per second: {:.2}", crypto_successful as f64 / crypto_duration.as_secs_f64());
            info!("  Average latency: {:.2} μs", crypto_avg_latency);
            info!("  Min latency: {} μs", if crypto_min_latency == u64::MAX { 0 } else { crypto_min_latency });
            info!("  Max latency: {} μs", crypto_max_latency);
        }
    }

    let total_duration = overall_start.elapsed();
    info!("\nTotal benchmark duration: {:?}", total_duration);
    
    Ok(())
}