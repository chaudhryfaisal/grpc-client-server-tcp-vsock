//! Benchmark tool for testing gRPC performance optimizations

use clap::{Arg, Command};
use grpc_performance_rs::transport::TransportConfig;
use grpc_performance_rs::{
    create_transport_channel,
    crypto::{crypto_service_client::CryptoServiceClient, KeyType, SignRequest, SigningAlgorithm},
    current_timestamp_millis,
    echo::{echo_service_client::EchoServiceClient, EchoRequest},
    AppResult, DEFAULT_SERVER_ADDR,
};
use log::{error, info};
use std::env;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use tokio::time::sleep;
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
        self.total_latency_micros
            .fetch_add(latency_micros, Ordering::Relaxed);

        // Update min latency
        let mut current_min = self.min_latency_micros.load(Ordering::Relaxed);
        while latency_micros < current_min {
            match self.min_latency_micros.compare_exchange_weak(
                current_min,
                latency_micros,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(x) => current_min = x,
            }
        }

        // Update max latency
        let mut current_max = self.max_latency_micros.load(Ordering::Relaxed);
        while latency_micros > current_max {
            match self.max_latency_micros.compare_exchange_weak(
                current_max,
                latency_micros,
                Ordering::Relaxed,
                Ordering::Relaxed,
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

        (
            total,
            successful,
            failed,
            avg_latency,
            min_latency,
            max_latency,
        )
    }
}

#[derive(Debug, Clone)]
struct BenchmarkConfig {
    connections: usize,
    threads: usize,
    requests: usize,
    rate_limit: Option<u64>,
    service: String,
    duration: Option<u64>,
    server_addr: String,
}

impl BenchmarkConfig {
    fn from_args_and_env() -> Result<Self, Box<dyn std::error::Error>> {
        let matches = Command::new("benchmark")
            .version("1.0")
            .about("gRPC benchmark tool supporting TCP and VSOCK transports")
            .arg(
                Arg::new("connections")
                    .long("connections")
                    .value_name("NUM")
                    .help("Number of concurrent connections")
                    .value_parser(clap::value_parser!(usize)),
            )
            .arg(
                Arg::new("threads")
                    .long("threads")
                    .value_name("NUM")
                    .help("Number of worker threads")
                    .value_parser(clap::value_parser!(usize)),
            )
            .arg(
                Arg::new("requests")
                    .long("requests")
                    .value_name("NUM")
                    .help("Total number of requests to send")
                    .value_parser(clap::value_parser!(usize)),
            )
            .arg(
                Arg::new("rate")
                    .long("rate")
                    .value_name("RPS")
                    .help("Requests per second limit (optional)")
                    .value_parser(clap::value_parser!(u64)),
            )
            .arg(
                Arg::new("service")
                    .long("service")
                    .value_name("TYPE")
                    .help("Service type: echo, rsa_sign, ecc_sign, or all")
                    .value_parser(["echo", "rsa_sign", "ecc_sign", "all"]),
            )
            .arg(
                Arg::new("duration")
                    .long("duration")
                    .value_name("DURATION")
                    .help("Test duration (e.g., 30s, 2m, 1h)"),
            )
            .arg(
                Arg::new("server")
                    .long("server")
                    .value_name("ADDR")
                    .help("Server address (overrides SERVER_ADDR env var)"),
            )
            .get_matches();

        // CLI arguments take precedence over environment variables
        let connections = matches
            .get_one::<usize>("connections")
            .copied()
            .or_else(|| {
                env::var("CONCURRENT_REQUESTS")
                    .ok()
                    .and_then(|s| s.parse().ok())
            })
            .or_else(|| env::var("CONNECTIONS").ok().and_then(|s| s.parse().ok()))
            .unwrap_or(10);

        let threads = matches
            .get_one::<usize>("threads")
            .copied()
            .or_else(|| env::var("THREADS").ok().and_then(|s| s.parse().ok()))
            .unwrap_or(4);

        let requests = matches
            .get_one::<usize>("requests")
            .copied()
            .or_else(|| env::var("TOTAL_REQUESTS").ok().and_then(|s| s.parse().ok()))
            .or_else(|| env::var("REQUESTS").ok().and_then(|s| s.parse().ok()))
            .unwrap_or(1000);

        let rate_limit = matches
            .get_one::<u64>("rate")
            .copied()
            .or_else(|| env::var("RATE_LIMIT").ok().and_then(|s| s.parse().ok()));

        let service = matches
            .get_one::<String>("service")
            .cloned()
            .or_else(|| env::var("BENCHMARK_TYPE").ok())
            .or_else(|| env::var("SERVICE").ok())
            .unwrap_or_else(|| "echo".to_string());

        // Parse duration string (e.g., "30s", "2m", "1h")
        let duration = matches
            .get_one::<String>("duration")
            .and_then(|s| parse_duration(s))
            .or_else(|| env::var("DURATION").ok().and_then(|s| s.parse().ok()));

        let server_addr = matches
            .get_one::<String>("server")
            .cloned()
            .or_else(|| env::var("SERVER_ADDR").ok())
            .unwrap_or_else(|| DEFAULT_SERVER_ADDR.to_string());

        if !["echo", "rsa_sign", "ecc_sign", "all"].contains(&service.as_str()) {
            return Err(format!(
                "Invalid service '{}'. Must be 'echo', 'rsa_sign', 'ecc_sign', or 'all'",
                service
            )
            .into());
        }

        if connections == 0 {
            return Err("Number of connections must be greater than 0".into());
        }

        if threads == 0 {
            return Err("Number of threads must be greater than 0".into());
        }

        // For duration-based benchmarks, requests parameter is ignored
        // For count-based benchmarks, requests must be > 0
        if duration.is_none() && requests == 0 {
            return Err(
                "For count-based benchmarks, requests must be greater than 0. Use --duration for time-based benchmarks.".into(),
            );
        }

        Ok(BenchmarkConfig {
            connections,
            threads,
            requests,
            rate_limit,
            service,
            duration,
            server_addr,
        })
    }
}

fn parse_duration(duration_str: &str) -> Option<u64> {
    let duration_str = duration_str.trim();

    if duration_str.ends_with('s') {
        duration_str[..duration_str.len() - 1].parse().ok()
    } else if duration_str.ends_with('m') {
        duration_str[..duration_str.len() - 1]
            .parse::<u64>()
            .ok()
            .map(|m| m * 60)
    } else if duration_str.ends_with('h') {
        duration_str[..duration_str.len() - 1]
            .parse::<u64>()
            .ok()
            .map(|h| h * 3600)
    } else {
        duration_str.parse().ok()
    }
}

/// Create a pool of reusable channels for efficient connection management
async fn create_channel_pool(addr: &str, pool_size: usize) -> AppResult<Vec<Channel>> {
    let transport_config = TransportConfig::from_str(&addr).map_err(|e| {
        error!("Invalid server address '{}': {}", addr, e);
        std::io::Error::new(std::io::ErrorKind::InvalidInput, e)
    })?;

    info!(
        "Creating channel pool: {} channels to {} ({})",
        pool_size,
        transport_config,
        if transport_config.is_tcp() {
            "TCP"
        } else {
            "VSOCK"
        }
    );

    let mut channels = Vec::with_capacity(pool_size);
    for _ in 0..pool_size {
        let channel = create_transport_channel(&transport_config).await?;
        channels.push(channel);
    }

    Ok(channels)
}

/// Service type for unified benchmark function
#[derive(Clone, Copy)]
enum ServiceType {
    Echo,
    RsaSign,
    EccSign,
}

/// Execute a single request for the specified service type
async fn execute_request(
    service_type: ServiceType,
    channel: Channel,
    request_id: u64,
) -> Result<u64, ()> {
    let start_time = Instant::now();

    let result = match service_type {
        ServiceType::Echo => {
            let mut client = EchoServiceClient::new(channel);
            let request = EchoRequest {
                payload: format!("Benchmark request {}", request_id),
                timestamp: current_timestamp_millis(),
            };
            client.echo(request).await.map(|_| ())
        }
        ServiceType::RsaSign => {
            let mut client = CryptoServiceClient::new(channel);
            let request = SignRequest {
                data: format!("Benchmark data {}", request_id).into_bytes(),
                key_type: KeyType::Rsa as i32,
                algorithm: SigningAlgorithm::RsaPkcs1Sha256 as i32,
                timestamp: current_timestamp_millis(),
            };
            client.sign(request).await.map(|_| ())
        }
        ServiceType::EccSign => {
            let mut client = CryptoServiceClient::new(channel);
            let request = SignRequest {
                data: format!("Benchmark data {}", request_id).into_bytes(),
                key_type: KeyType::Ecc as i32,
                algorithm: SigningAlgorithm::EcdsaP256Sha256 as i32,
                timestamp: current_timestamp_millis(),
            };
            client.sign(request).await.map(|_| ())
        }
    };

    match result {
        Ok(_) => Ok(start_time.elapsed().as_micros() as u64),
        Err(_) => Err(()),
    }
}

/// Simplified unified benchmark function
async fn run_benchmark(
    service_type: ServiceType,
    channels: Arc<Vec<Channel>>,
    concurrent_requests: usize,
    total_requests: usize,
    metrics: BenchmarkMetrics,
    rate_limit: Option<u64>,
    duration: Option<u64>,
) -> AppResult<()> {
    let service_name = match service_type {
        ServiceType::Echo => "echo",
        ServiceType::RsaSign => "rsa_sign",
        ServiceType::EccSign => "ecc_sign",
    };

    info!(
        "Starting {} service benchmark: {} concurrent connections",
        service_name, concurrent_requests
    );

    let semaphore = Arc::new(Semaphore::new(concurrent_requests));
    let stop_flag = Arc::new(AtomicBool::new(false));
    let request_counter = Arc::new(AtomicU64::new(0));

    // Start duration timer if specified
    if let Some(duration_secs) = duration {
        let stop_flag_clone = stop_flag.clone();
        tokio::spawn(async move {
            sleep(Duration::from_secs(duration_secs)).await;
            stop_flag_clone.store(true, Ordering::Relaxed);
        });
    }

    // Calculate rate limiting interval (only apply if explicitly specified)
    let rate_interval = rate_limit.map(|rps| Duration::from_nanos(1_000_000_000 / rps));

    let mut tasks = Vec::new();

    // For duration-based benchmarks, spawn concurrent workers that run continuously
    if duration.is_some() {
        // Spawn concurrent workers that continuously make requests
        for _i in 0..concurrent_requests {
            let channels = channels.clone();
            let stop_flag = stop_flag.clone();
            let request_counter = request_counter.clone();
            let metrics = metrics.clone();
            let rate_interval = rate_interval;

            let task = tokio::spawn(async move {
                let mut last_request_time = Instant::now();
                
                while !stop_flag.load(Ordering::Relaxed) {
                    // Apply rate limiting per worker if specified
                    if let Some(interval) = rate_interval {
                        let worker_interval = Duration::from_nanos(interval.as_nanos() as u64 * concurrent_requests as u64);
                        let next_request_time = last_request_time + worker_interval;
                        let now = Instant::now();
                        if now < next_request_time {
                            sleep(next_request_time - now).await;
                        }
                        last_request_time = Instant::now();
                    }

                    // Get next request ID and select channel
                    let request_id = request_counter.fetch_add(1, Ordering::Relaxed);
                    let channel_index = request_id as usize % channels.len();
                    let channel = channels[channel_index].clone();
                    
                    match execute_request(service_type, channel, request_id).await {
                        Ok(latency) => metrics.record_success(latency),
                        Err(_) => metrics.record_failure(),
                    }
                }
            });

            tasks.push(task);
        }
    } else {
        // For count-based benchmarks, use the original approach but without artificial rate limiting
        let mut last_request_time = Instant::now();

        // Main request loop for count-based benchmarks
        for request_id in 0..total_requests {
            // Check if we should stop
            if stop_flag.load(Ordering::Relaxed) {
                break;
            }

            // Apply rate limiting only if explicitly specified
            if let Some(interval) = rate_interval {
                let next_request_time = last_request_time + interval;
                let now = Instant::now();
                if now < next_request_time {
                    sleep(next_request_time - now).await;
                }
                last_request_time = Instant::now();
            }

            // Select channel
            let channel_index = request_id % channels.len();
            let channel = channels[channel_index].clone();
            
            // Clone shared resources for the task
            let semaphore = semaphore.clone();
            let metrics = metrics.clone();

            let task = tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                
                match execute_request(service_type, channel, request_id as u64).await {
                    Ok(latency) => metrics.record_success(latency),
                    Err(_) => metrics.record_failure(),
                }
            });

            tasks.push(task);
        }
    }

    // Wait for all tasks to complete
    for task in tasks {
        let _ = task.await;
    }

    Ok(())
}

fn print_results(service_name: &str, metrics: &BenchmarkMetrics, duration: Duration) {
    let (total, successful, failed, avg_latency, min_latency, max_latency) = metrics.get_stats();

    info!("\n=== {} Service Benchmark Results ===", service_name);
    info!("Total requests: {}", total);
    info!("Successful requests: {}", successful);
    info!("Failed requests: {}", failed);
    info!(
        "Success rate: {:.2}%",
        if total > 0 {
            (successful as f64 / total as f64) * 100.0
        } else {
            0.0
        }
    );
    info!("Duration: {:?}", duration);
    info!(
        "Requests per second: {:.2}",
        successful as f64 / duration.as_secs_f64()
    );
    info!("Average latency: {:.2} μs", avg_latency);
    info!(
        "Min latency: {} μs",
        if min_latency == u64::MAX {
            0
        } else {
            min_latency
        }
    );
    info!("Max latency: {} μs", max_latency);
}

#[tokio::main]
async fn main() -> AppResult<()> {
    // Initialize logging
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    // Parse configuration from CLI args and environment variables
    let config = match BenchmarkConfig::from_args_and_env() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Configuration error: {}", e);
            std::process::exit(1);
        }
    };

    info!("Starting gRPC performance benchmark");
    info!("Server: {}", config.server_addr);
    info!("Concurrent connections: {}", config.connections);
    info!("Worker threads: {}", config.threads);
    
    if let Some(duration) = config.duration {
        info!("Duration: {} seconds", duration);
    } else {
        info!("Total requests: {}", config.requests);
    }
    
    if let Some(rate) = config.rate_limit {
        info!("Rate limit: {} requests/second", rate);
    }
    info!("Service: {}", config.service);

    // Create channel pool for connection reuse
    let channels = Arc::new(
        create_channel_pool(&config.server_addr, config.connections).await?
    );

    // Helper function to get service display name
    fn get_service_display_name(service_type: ServiceType) -> &'static str {
        match service_type {
            ServiceType::Echo => "Echo",
            ServiceType::RsaSign => "RSA Sign",
            ServiceType::EccSign => "ECC Sign",
        }
    }

    // Helper function to run a single service benchmark
    async fn run_single_service_benchmark(
        service_type: ServiceType,
        channels: Arc<Vec<Channel>>,
        config: &BenchmarkConfig,
    ) -> AppResult<()> {
        let metrics = BenchmarkMetrics::new();
        let start_time = Instant::now();

        run_benchmark(
            service_type,
            channels,
            config.connections,
            config.requests,
            metrics.clone(),
            config.rate_limit,
            config.duration,
        )
        .await?;

        let duration = start_time.elapsed();
        print_results(get_service_display_name(service_type), &metrics, duration);
        Ok(())
    }

    let overall_start = Instant::now();

    // Determine which services to benchmark
    let services_to_benchmark = match config.service.as_str() {
        "echo" => vec![ServiceType::Echo],
        "rsa_sign" => vec![ServiceType::RsaSign],
        "ecc_sign" => vec![ServiceType::EccSign],
        "all" => vec![ServiceType::Echo, ServiceType::RsaSign, ServiceType::EccSign],
        _ => unreachable!(), // Already validated above
    };

    // Run benchmarks for each service
    for service_type in services_to_benchmark {
        println!("\n=== {} Service Benchmark ===", get_service_display_name(service_type));
        run_single_service_benchmark(service_type, channels.clone(), &config).await?;
    }

    // If benchmarking all services, print comparison summary
    if config.service == "all" {
        println!("\n=== Service Comparison Summary ===");
        println!("All Echo, RSA Sign, and ECC Sign services completed successfully");
        println!("See individual results above for detailed metrics");
    }

    let total_duration = overall_start.elapsed();
    info!("\nTotal benchmark duration: {:?}", total_duration);

    Ok(())
}
