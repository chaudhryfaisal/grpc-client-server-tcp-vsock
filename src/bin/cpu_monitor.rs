use log::info;
use std::collections::VecDeque;
use std::{env, thread, time::Duration};
use sysinfo::System;

fn percentile(sorted: &[f32], p: f32) -> f32 {
    if sorted.is_empty() {
        return 0.0;
    }
    let rank = (p * sorted.len() as f32).ceil() as usize - 1;
    sorted[rank.min(sorted.len() - 1)]
}

fn get_env_or<T: std::str::FromStr>(key: &str, default: T) -> T {
    env::var(key)
        .ok()
        .and_then(|v| v.parse::<T>().ok())
        .unwrap_or(default)
}

fn main() {
    // Init logger with default level = info
    if env::var("RUST_LOG").is_err() {
        unsafe { env::set_var("RUST_LOG", "info") }
    }
    env_logger::init();

    let sample_interval = get_env_or("CPU_MONITOR_INTERVAL", 1); // seconds
    let summary_period = get_env_or("CPU_MONITOR_SUMMARY", 5); // seconds
    let window_size = summary_period / sample_interval;

    info!(
        "Starting CPU monitor: sample_interval={}s, summary_period={}s (window_size={})",
        sample_interval, summary_period, window_size
    );

    let mut sys = System::new_all();
    let mut samples: VecDeque<f32> = VecDeque::with_capacity(window_size);

    loop {
        sys.refresh_cpu_all();
        let avg_usage: f32 =
            sys.cpus().iter().map(|cpu| cpu.cpu_usage()).sum::<f32>() / sys.cpus().len() as f32;

        if samples.len() == window_size {
            samples.pop_front();
        }
        samples.push_back(avg_usage);

        if samples.len() == window_size {
            let mut sorted = samples.iter().cloned().collect::<Vec<_>>();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

            let min = *sorted.first().unwrap();
            let max = *sorted.last().unwrap();
            let avg = sorted.iter().sum::<f32>() / sorted.len() as f32;
            let p95 = percentile(&sorted, 0.95);
            let p99 = percentile(&sorted, 0.99);

            info!(
                "min={:.2}% max={:.2}% avg={:.2}% p95={:.2}% p99={:.2}%",
                min, max, avg, p95, p99
            );
        }

        thread::sleep(Duration::from_secs(sample_interval as u64));
    }
}
