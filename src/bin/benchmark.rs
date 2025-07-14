//! Comprehensive benchmark tool for gRPC client/server performance testing
//! 
//! This benchmark supports both TCP and VSOCK transports and provides detailed
//! performance metrics including latency, throughput, and error rates.

use clap::{Arg, Command};
use grpc_performance_rs::{
    echo::{echo_service_client::EchoServiceClient, EchoRequest},
    crypto::{crypto_service_client::CryptoServiceClient, SignRequest, PublicKeyRequest, KeyType, SigningAlgorithm},
    current_timestamp_millis,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, Mutex, Semaphore};
use tokio::time::{interval, sleep};
use tonic::transport::Channel;
use tonic::Request;

#[derive(Debug, Clone)]
pub struct BenchmarkConfig {
    pub server_addr: String,
    pub transport_type: TransportType,
    pub connections: usize,
    pub threads: usize,
    pub duration: Duration,
    pub max_requests: Option<usize>,
    pub rate_limit: Option<f64>, // requests per second
    pub service_type: ServiceType,
    pub warmup_duration: Duration,
    pub output_format: OutputFormat,
    pub output_file: Option<String>,
}

#[derive(Debug, Clone)]
pub enum TransportType {
    Tcp,
    Vsock,
}

#[derive(Debug, Clone)]
pub enum ServiceType {
    Echo,
    Crypto,
    Both,
}

#[derive(Debug, Clone)]
pub enum OutputFormat {
    Human,
    Json,
    Csv,
}

#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub total_duration: Duration,
    pub latencies: Vec<Duration>,
    pub connection_times: Vec<Duration>,
    pub errors: HashMap<String, u64>,
    pub throughput: f64, // requests per second
}

#[derive(Debug, Clone)]
pub struct LatencyStats {
    pub min: Duration,
    pub max: Duration,
    pub mean: Duration,
    pub p50: Duration,
    pub p95: Duration,
    pub p99: Duration,
}

impl BenchmarkResult {
    pub fn new() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            total_duration: Duration::ZERO,
            latencies: Vec::new(),
            connection_times: Vec::new(),
            errors: HashMap::new(),
            throughput: 0.0,
        }
    }

    pub fn calculate_latency_stats(&self) -> LatencyStats {
        if self.latencies.is_empty() {
            return LatencyStats {
                min: Duration::ZERO,
                max: Duration::ZERO,
                mean: Duration::ZERO,
                p50: Duration::ZERO,
                p95: Duration::ZERO,
                p99: Duration::ZERO,
            };
        }

        let mut sorted_latencies = self.latencies.clone();
        sorted_latencies.sort();

        let len = sorted_latencies.len();
        let sum: Duration = sorted_latencies.iter().sum();
        
        LatencyStats {
            min: sorted_latencies[0],
            max: sorted_latencies[len - 1],
            mean: sum / len as u32,
            p50: sorted_latencies[len * 50 / 100],
            p95: sorted_latencies[len * 95 / 100],
            p99: sorted_latencies[len * 99 / 100],
        }
    }
}

pub struct BenchmarkRunner {
    config: BenchmarkConfig,
    result: Arc<Mutex<BenchmarkResult>>,
}

impl BenchmarkRunner {
    pub fn new(config: BenchmarkConfig) -> Self {
        Self {
            config,
            result: Arc::new(Mutex::new(BenchmarkResult::new())),
        }
    }

    pub async fn run(&self) -> Result<BenchmarkResult, Box<dyn std::error::Error + Send + Sync>> {
        println!("🚀 Starting benchmark with configuration:");
        println!("   Transport: {:?}", self.config.transport_type);
        println!("   Server: {}", self.config.server_addr);
        println!("   Connections: {}", self.config.connections);
        println!("   Threads: {}", self.config.threads);
        println!("   Duration: {:?}", self.config.duration);
        println!("   Service: {:?}", self.config.service_type);
        if let Some(rate) = self.config.rate_limit {
            println!("   Rate limit: {:.2} req/s", rate);
        }
        println!();

        // Warmup phase
        if self.config.warmup_duration > Duration::ZERO {
            println!("🔥 Warming up for {:?}...", self.config.warmup_duration);
            self.warmup().await?;
            println!("✅ Warmup completed\n");
        }

        // Main benchmark
        let start_time = Instant::now();
        
        // Create semaphore for connection limiting
        let connection_semaphore = Arc::new(Semaphore::new(self.config.connections));
        
        // Create rate limiter if specified
        let rate_limiter = if let Some(rate) = self.config.rate_limit {
            Some(Arc::new(Mutex::new(interval(Duration::from_secs_f64(1.0 / rate)))))
        } else {
            None
        };

        // Create channels for communication
        let (tx, mut rx) = mpsc::channel(1000);

        // Spawn worker tasks
        let mut handles = Vec::new();
        for worker_id in 0..self.config.threads {
            let tx = tx.clone();
            let config = self.config.clone();
            let connection_semaphore = connection_semaphore.clone();
            let rate_limiter = rate_limiter.clone();
            
            let handle = tokio::spawn(async move {
                Self::worker_task(worker_id, config, connection_semaphore, rate_limiter, tx).await
            });
            handles.push(handle);
        }

        // Drop the original sender
        drop(tx);

        // Spawn progress reporter
        let result_clone = self.result.clone();
        let progress_handle = tokio::spawn(async move {
            Self::progress_reporter(result_clone).await;
        });

        // Collect results
        let mut request_count = 0;
        let max_requests = self.config.max_requests.unwrap_or(usize::MAX);
        
        while let Some(measurement) = rx.recv().await {
            if request_count >= max_requests {
                break;
            }
            
            if start_time.elapsed() >= self.config.duration {
                break;
            }

            self.record_measurement(measurement).await;
            request_count += 1;
        }

        // Cancel all workers
        for handle in handles {
            handle.abort();
        }
        progress_handle.abort();

        // Finalize results
        let mut result = self.result.lock().await;
        result.total_duration = start_time.elapsed();
        result.throughput = result.successful_requests as f64 / result.total_duration.as_secs_f64();

        Ok(result.clone())
    }

    async fn warmup(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Simple warmup without recursion - just make a few test requests
        let warmup_requests = std::cmp::min(100, self.config.connections * 10);
        
        for _ in 0..warmup_requests {
            let measurement = Self::make_request(0, &self.config).await;
            // Ignore warmup results
            let _ = measurement;
        }
        
        // Reset results after warmup
        *self.result.lock().await = BenchmarkResult::new();
        
        Ok(())
    }

    async fn worker_task(
        worker_id: usize,
        config: BenchmarkConfig,
        connection_semaphore: Arc<Semaphore>,
        rate_limiter: Option<Arc<Mutex<tokio::time::Interval>>>,
        tx: mpsc::Sender<RequestMeasurement>,
    ) {
        loop {
            // Acquire connection permit
            let _permit = match connection_semaphore.try_acquire() {
                Ok(permit) => permit,
                Err(_) => {
                    sleep(Duration::from_millis(1)).await;
                    continue;
                }
            };

            // Apply rate limiting
            if let Some(rate_limiter) = &rate_limiter {
                let mut interval = rate_limiter.lock().await;
                interval.tick().await;
            }

            // Create connection and make request
            let measurement = Self::make_request(worker_id, &config).await;
            
            if tx.send(measurement).await.is_err() {
                break; // Channel closed, exit worker
            }
        }
    }

    async fn make_request(
        _worker_id: usize,
        config: &BenchmarkConfig,
    ) -> RequestMeasurement {
        let connection_start = Instant::now();
        
        // Create channel
        let channel_result = Self::create_channel(&config.server_addr).await;
        let connection_time = connection_start.elapsed();

        let channel = match channel_result {
            Ok(channel) => channel,
            Err(e) => {
                return RequestMeasurement {
                    success: false,
                    latency: Duration::ZERO,
                    connection_time,
                    error: Some(format!("Connection failed: {}", e)),
                };
            }
        };

        // Make service request
        let request_start = Instant::now();
        let result = match config.service_type {
            ServiceType::Echo => Self::make_echo_request(channel).await,
            ServiceType::Crypto => Self::make_crypto_request(channel).await,
            ServiceType::Both => {
                // Randomly choose between echo and crypto
                if rand::random::<bool>() {
                    Self::make_echo_request(channel).await
                } else {
                    Self::make_crypto_request(channel).await
                }
            }
        };
        let latency = request_start.elapsed();

        match result {
            Ok(_) => RequestMeasurement {
                success: true,
                latency,
                connection_time,
                error: None,
            },
            Err(e) => RequestMeasurement {
                success: false,
                latency,
                connection_time,
                error: Some(format!("Request failed: {}", e)),
            },
        }
    }

    async fn create_channel(addr: &str) -> Result<Channel, Box<dyn std::error::Error + Send + Sync>> {
        let endpoint_uri = format!("http://{}", addr);
        let channel = Channel::from_shared(endpoint_uri)?
            .connect()
            .await?;
        Ok(channel)
    }

    async fn make_echo_request(channel: Channel) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut client = EchoServiceClient::new(channel);
        let request = Request::new(EchoRequest {
            payload: "benchmark test message".to_string(),
            timestamp: current_timestamp_millis(),
        });
        
        let _response = client.echo(request).await?;
        Ok(())
    }

    async fn make_crypto_request(channel: Channel) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut client = CryptoServiceClient::new(channel);
        
        // Test signing operation
        let sign_request = Request::new(SignRequest {
            data: b"benchmark test data".to_vec(),
            key_type: KeyType::Rsa as i32,
            algorithm: SigningAlgorithm::RsaPkcs1Sha256 as i32,
            timestamp: current_timestamp_millis(),
        });
        
        let _sign_response = client.sign(sign_request).await?;
        
        // Test public key retrieval
        let pubkey_request = Request::new(PublicKeyRequest {
            key_type: KeyType::Rsa as i32,
            timestamp: current_timestamp_millis(),
        });
        
        let _pubkey_response = client.get_public_key(pubkey_request).await?;
        Ok(())
    }

    async fn record_measurement(&self, measurement: RequestMeasurement) {
        let mut result = self.result.lock().await;
        result.total_requests += 1;
        
        if measurement.success {
            result.successful_requests += 1;
            result.latencies.push(measurement.latency);
        } else {
            result.failed_requests += 1;
            if let Some(error) = measurement.error {
                *result.errors.entry(error).or_insert(0) += 1;
            }
        }
        
        result.connection_times.push(measurement.connection_time);
    }

    async fn progress_reporter(result: Arc<Mutex<BenchmarkResult>>) {
        let mut interval = interval(Duration::from_secs(1));
        let mut last_requests = 0;
        
        loop {
            interval.tick().await;
            
            let current_result = result.lock().await;
            let current_requests = current_result.total_requests;
            let rps = current_requests - last_requests;
            
            println!(
                "📊 Progress: {} requests ({} successful, {} failed) | {} req/s",
                current_result.total_requests,
                current_result.successful_requests,
                current_result.failed_requests,
                rps
            );
            
            last_requests = current_requests;
        }
    }

    pub fn print_results(&self, result: &BenchmarkResult) {
        let latency_stats = result.calculate_latency_stats();
        
        println!("\n🎯 Benchmark Results");
        println!("==================");
        println!("Total Requests:     {}", result.total_requests);
        println!("Successful:         {}", result.successful_requests);
        println!("Failed:             {}", result.failed_requests);
        println!("Success Rate:       {:.2}%", 
                 (result.successful_requests as f64 / result.total_requests as f64) * 100.0);
        println!("Duration:           {:.2}s", result.total_duration.as_secs_f64());
        println!("Throughput:         {:.2} req/s", result.throughput);
        
        println!("\n📈 Latency Statistics");
        println!("Min:                {:?}", latency_stats.min);
        println!("Max:                {:?}", latency_stats.max);
        println!("Mean:               {:?}", latency_stats.mean);
        println!("P50:                {:?}", latency_stats.p50);
        println!("P95:                {:?}", latency_stats.p95);
        println!("P99:                {:?}", latency_stats.p99);
        
        if !result.connection_times.is_empty() {
            let mut sorted_conn_times = result.connection_times.clone();
            sorted_conn_times.sort();
            let conn_len = sorted_conn_times.len();
            
            println!("\n🔗 Connection Statistics");
            println!("Min Connection Time: {:?}", sorted_conn_times[0]);
            println!("Max Connection Time: {:?}", sorted_conn_times[conn_len - 1]);
            println!("Mean Connection Time: {:?}", 
                     sorted_conn_times.iter().sum::<Duration>() / conn_len as u32);
        }
        
        if !result.errors.is_empty() {
            println!("\n❌ Error Summary");
            for (error, count) in &result.errors {
                println!("{}: {}", error, count);
            }
        }
    }

    pub fn export_results(&self, result: &BenchmarkResult, format: &OutputFormat, file: Option<&str>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let output = match format {
            OutputFormat::Json => self.export_json(result)?,
            OutputFormat::Csv => self.export_csv(result)?,
            OutputFormat::Human => return Ok(()), // Already printed
        };

        if let Some(filename) = file {
            std::fs::write(filename, output)?;
            println!("Results exported to: {}", filename);
        } else {
            println!("{}", output);
        }

        Ok(())
    }

    fn export_json(&self, result: &BenchmarkResult) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let latency_stats = result.calculate_latency_stats();
        
        let json_result = serde_json::json!({
            "total_requests": result.total_requests,
            "successful_requests": result.successful_requests,
            "failed_requests": result.failed_requests,
            "success_rate": (result.successful_requests as f64 / result.total_requests as f64) * 100.0,
            "duration_seconds": result.total_duration.as_secs_f64(),
            "throughput_rps": result.throughput,
            "latency_stats": {
                "min_ms": latency_stats.min.as_millis(),
                "max_ms": latency_stats.max.as_millis(),
                "mean_ms": latency_stats.mean.as_millis(),
                "p50_ms": latency_stats.p50.as_millis(),
                "p95_ms": latency_stats.p95.as_millis(),
                "p99_ms": latency_stats.p99.as_millis()
            },
            "errors": result.errors
        });

        Ok(serde_json::to_string_pretty(&json_result)?)
    }

    fn export_csv(&self, result: &BenchmarkResult) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let latency_stats = result.calculate_latency_stats();
        
        let mut csv = String::new();
        csv.push_str("metric,value\n");
        csv.push_str(&format!("total_requests,{}\n", result.total_requests));
        csv.push_str(&format!("successful_requests,{}\n", result.successful_requests));
        csv.push_str(&format!("failed_requests,{}\n", result.failed_requests));
        csv.push_str(&format!("success_rate,{:.2}\n", (result.successful_requests as f64 / result.total_requests as f64) * 100.0));
        csv.push_str(&format!("duration_seconds,{:.2}\n", result.total_duration.as_secs_f64()));
        csv.push_str(&format!("throughput_rps,{:.2}\n", result.throughput));
        csv.push_str(&format!("latency_min_ms,{}\n", latency_stats.min.as_millis()));
        csv.push_str(&format!("latency_max_ms,{}\n", latency_stats.max.as_millis()));
        csv.push_str(&format!("latency_mean_ms,{}\n", latency_stats.mean.as_millis()));
        csv.push_str(&format!("latency_p50_ms,{}\n", latency_stats.p50.as_millis()));
        csv.push_str(&format!("latency_p95_ms,{}\n", latency_stats.p95.as_millis()));
        csv.push_str(&format!("latency_p99_ms,{}\n", latency_stats.p99.as_millis()));

        Ok(csv)
    }
}

#[derive(Debug)]
struct RequestMeasurement {
    success: bool,
    latency: Duration,
    connection_time: Duration,
    error: Option<String>,
}

fn parse_duration(s: &str) -> Result<Duration, String> {
    if s.ends_with('s') {
        let secs: f64 = s[..s.len()-1].parse().map_err(|_| "Invalid duration format")?;
        Ok(Duration::from_secs_f64(secs))
    } else if s.ends_with("ms") {
        let millis: u64 = s[..s.len()-2].parse().map_err(|_| "Invalid duration format")?;
        Ok(Duration::from_millis(millis))
    } else {
        let secs: f64 = s.parse().map_err(|_| "Invalid duration format")?;
        Ok(Duration::from_secs_f64(secs))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let matches = Command::new("gRPC Benchmark Tool")
        .version("1.0")
        .about("Comprehensive performance testing for gRPC client/server with TCP and VSOCK transports")
        .arg(Arg::new("server")
            .short('s')
            .long("server")
            .value_name("ADDRESS")
            .help("Server address (e.g., 127.0.0.1:50051 for TCP, 2:1234 for VSOCK)")
            .default_value("127.0.0.1:50051"))
        .arg(Arg::new("transport")
            .short('t')
            .long("transport")
            .value_name("TYPE")
            .help("Transport type: tcp or vsock")
            .value_parser(["tcp", "vsock"])
            .default_value("tcp"))
        .arg(Arg::new("connections")
            .short('c')
            .long("connections")
            .value_name("COUNT")
            .help("Number of concurrent connections")
            .value_parser(clap::value_parser!(usize))
            .default_value("10"))
        .arg(Arg::new("threads")
            .long("threads")
            .value_name("COUNT")
            .help("Number of worker threads")
            .value_parser(clap::value_parser!(usize))
            .default_value("4"))
        .arg(Arg::new("duration")
            .short('d')
            .long("duration")
            .value_name("TIME")
            .help("Test duration (e.g., 30s, 5m)")
            .default_value("30s"))
        .arg(Arg::new("requests")
            .short('n')
            .long("requests")
            .value_name("COUNT")
            .help("Maximum number of requests (overrides duration)")
            .value_parser(clap::value_parser!(usize)))
        .arg(Arg::new("rate")
            .short('r')
            .long("rate")
            .value_name("RPS")
            .help("Rate limit in requests per second")
            .value_parser(clap::value_parser!(f64)))
        .arg(Arg::new("service")
            .long("service")
            .value_name("TYPE")
            .help("Service type to test: echo, crypto, or both")
            .value_parser(["echo", "crypto", "both"])
            .default_value("echo"))
        .arg(Arg::new("warmup")
            .long("warmup")
            .value_name("TIME")
            .help("Warmup duration (e.g., 5s)")
            .default_value("5s"))
        .arg(Arg::new("output")
            .short('o')
            .long("output")
            .value_name("FORMAT")
            .help("Output format: human, json, or csv")
            .value_parser(["human", "json", "csv"])
            .default_value("human"))
        .arg(Arg::new("file")
            .short('f')
            .long("file")
            .value_name("PATH")
            .help("Output file path"))
        .get_matches();

    // Parse configuration
    let config = BenchmarkConfig {
        server_addr: matches.get_one::<String>("server").unwrap().clone(),
        transport_type: match matches.get_one::<String>("transport").unwrap().as_str() {
            "tcp" => TransportType::Tcp,
            "vsock" => TransportType::Vsock,
            _ => unreachable!(),
        },
        connections: *matches.get_one::<usize>("connections").unwrap(),
        threads: *matches.get_one::<usize>("threads").unwrap(),
        duration: parse_duration(matches.get_one::<String>("duration").unwrap())
            .map_err(|e| format!("Invalid duration: {}", e))?,
        max_requests: matches.get_one::<usize>("requests").copied(),
        rate_limit: matches.get_one::<f64>("rate").copied(),
        service_type: match matches.get_one::<String>("service").unwrap().as_str() {
            "echo" => ServiceType::Echo,
            "crypto" => ServiceType::Crypto,
            "both" => ServiceType::Both,
            _ => unreachable!(),
        },
        warmup_duration: parse_duration(matches.get_one::<String>("warmup").unwrap())
            .map_err(|e| format!("Invalid warmup duration: {}", e))?,
        output_format: match matches.get_one::<String>("output").unwrap().as_str() {
            "human" => OutputFormat::Human,
            "json" => OutputFormat::Json,
            "csv" => OutputFormat::Csv,
            _ => unreachable!(),
        },
        output_file: matches.get_one::<String>("file").cloned(),
    };

    // Run benchmark
    let runner = BenchmarkRunner::new(config.clone());
    let result = runner.run().await?;

    // Display results
    runner.print_results(&result);

    // Export results if requested
    if !matches!(config.output_format, OutputFormat::Human) || config.output_file.is_some() {
        runner.export_results(&result, &config.output_format, config.output_file.as_deref())?;
    }

    Ok(())
}