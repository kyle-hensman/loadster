use clap::Parser;
use std::time::{Duration, Instant};
use std::sync::Arc;
use tokio::task::JoinSet;
use serde::{Serialize, Deserialize};
use std::fs;
use chrono::{DateTime, Utc};

const VERSION: &str = "1.0.0";

#[derive(Serialize, Deserialize, Debug)]
struct Report {
    url: String,
    date: DateTime<Utc>,
    total_requests: usize,
    concurrency: usize,
    total_duration_secs: f64,
    successful: usize,
    failed: usize,
    requests_per_sec: f64,
    latency: LatencyStats,
}

#[derive(Serialize, Deserialize, Debug)]
struct LatencyStats {
    avg_ms: f64,
    p50_ms: f64,
    p95_ms: f64,
    p99_ms: f64,
    min_ms: f64,
    max_ms: f64,
}

/// A simple HTTP load testing tool
#[derive(Parser, Debug)]
#[command(name = "loadster")]
#[command(author = "Kyle Hensman")]
#[command(version = VERSION)]
#[command(about = "Simple HTTP load testing CLI", long_about = "
A lightweight HTTP load testing tool that sends concurrent requests
and reports latency statistics including p50, p95, and p99 percentiles.

Example:
  loadster https://example.com -n 200 -c 20
")]
struct Args {
    /// URL to test (must include http:// or https://)
    #[arg(value_name = "URL")]
    url: String,

    /// Total number of requests to send
    #[arg(short = 'n', long, default_value = "100")]
    requests: usize,

    /// Number of requests to run concurrently
    #[arg(short = 'c', long, default_value = "10")]
    concurrency: usize,

    /// Output file path for JSON report (optional)
    #[arg(short = 'o', long, value_name = "FILE", default_value = "loadster-report.json")]
    output: Option<String>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    
    let url = &args.url;
    let total_requests = args.requests;
    let concurrency = args.concurrency;

    println!("Load testing: {}", url);
    println!("Total requests: {}", total_requests);
    println!("Concurrency: {}\n", concurrency);

    let client = Arc::new(reqwest::Client::new());
    let url = Arc::new(url.to_string());
    
    let start = Instant::now();
    let mut tasks = JoinSet::new();

    // Spawn all tasks
    for _ in 0..total_requests {
        let client = Arc::clone(&client);
        let url = Arc::clone(&url);
        
        tasks.spawn(async move {
            let req_start = Instant::now();
            let result = client.get(url.as_str()).send().await;
            let duration = req_start.elapsed();
            
            match result {
                Ok(resp) => (true, resp.status().as_u16(), duration),
                Err(_) => (false, 0, duration),
            }
        });

        // Limit active tasks to concurrency level
        while tasks.len() > concurrency {
            tasks.join_next().await;
        }
    }

    // Collect all results
    let mut success = 0;
    let mut failed = 0;
    let mut durations = Vec::new();
    let mut completed = 0;

    while let Some(result) = tasks.join_next().await {
        if let Ok((ok, _status, dur)) = result {
            if ok {
                success += 1;
                print!(".");
            } else {
                failed += 1;
                print!("F");
            }
            durations.push(dur);
            completed += 1;

            if completed % 50 == 0 {
                println!(" {}/{}", completed, total_requests);
            }
        }
    }

    if completed % 50 != 0 {
        println!();
    }

    let total_duration = start.elapsed();
    println!("\n\nResults:");
    println!("========");
    println!("Total time: {:.2}s", total_duration.as_secs_f64());
    println!("Successful: {}", success);
    println!("Failed: {}", failed);
    println!("Requests/sec: {:.2}", total_requests as f64 / total_duration.as_secs_f64());
    
    let mut latency_stats = None;
    
    if !durations.is_empty() {
        durations.sort();
        let avg: Duration = durations.iter().sum::<Duration>() / durations.len() as u32;
        let min = durations[0];
        let max = durations[durations.len() - 1];
        let p50 = durations[durations.len() / 2];
        let p95 = durations[durations.len() * 95 / 100];
        let p99 = durations[durations.len() * 99 / 100];
        
        println!("\nLatency:");
        println!("  Min: {:.2}ms", min.as_secs_f64() * 1000.0);
        println!("  Avg: {:.2}ms", avg.as_secs_f64() * 1000.0);
        println!("  p50: {:.2}ms", p50.as_secs_f64() * 1000.0);
        println!("  p95: {:.2}ms", p95.as_secs_f64() * 1000.0);
        println!("  p99: {:.2}ms", p99.as_secs_f64() * 1000.0);
        println!("  Max: {:.2}ms", max.as_secs_f64() * 1000.0);

        latency_stats = Some(LatencyStats {
            avg_ms: avg.as_secs_f64() * 1000.0,
            p50_ms: p50.as_secs_f64() * 1000.0,
            p95_ms: p95.as_secs_f64() * 1000.0,
            p99_ms: p99.as_secs_f64() * 1000.0,
            min_ms: min.as_secs_f64() * 1000.0,
            max_ms: max.as_secs_f64() * 1000.0,
        });
    }

    // Save JSON report if output path provided
    if let Some(output_path) = &args.output {
        let timestamp: DateTime<Utc> = Utc::now();

        let report = Report {
            url: url.to_string(),
            date: timestamp,
            total_requests,
            concurrency,
            total_duration_secs: total_duration.as_secs_f64(),
            successful: success,
            failed,
            requests_per_sec: total_requests as f64 / total_duration.as_secs_f64(),
            latency: latency_stats.unwrap_or(LatencyStats {
                avg_ms: 0.0,
                p50_ms: 0.0,
                p95_ms: 0.0,
                p99_ms: 0.0,
                min_ms: 0.0,
                max_ms: 0.0,
            }),
        };

        match fs::write(output_path, serde_json::to_string_pretty(&report).unwrap()) {
            Ok(_) => println!("\n✓ Report saved to: {}", output_path),
            Err(e) => eprintln!("\n✗ Failed to save report: {}", e),
        }
    }
}
