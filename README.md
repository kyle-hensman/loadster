# Loadster CLI

A lightweight, fast, and simple HTTP load testing tool written in Rust. Perfect for quickly stress testing your web applications and APIs with concurrent requests.

## Features

- 🚀 **Fast & Concurrent** - Leverages Rust's async capabilities with Tokio
- 📊 **Detailed Statistics** - Get latency percentiles (p50, p95, p99) and throughput metrics
- 💾 **JSON Reports** - Export results to JSON for further analysis
- 🎯 **Simple CLI** - Easy to use with sensible defaults
- 🔧 **Configurable** - Control request count and concurrency level
- ⚡ **Lightweight** - Small binary with minimal dependencies

## Installation

### From Source

```bash
git clone https://github.com/kyle-hensman/loadster
cd loadster
cargo install --path .
```

### From Crates.io

```bash
cargo install loadster
```

### Requirements

- Rust 1.70 or higher
- Cargo

## Usage

### Basic Usage

```bash
loadster https://example.com
```

This will send 100 requests with a concurrency of 10 (default values).

### Custom Request Count and Concurrency

```bash
# Send 500 requests with 50 concurrent connections
loadster https://example.com -n 500 -c 50

# Using long form flags
loadster https://example.com --requests 1000 --concurrency 100
```

### Save Results to JSON

```bash
# Save detailed report to a JSON file
loadster https://example.com -o report.json

# Or with long form
loadster https://example.com --output results.json