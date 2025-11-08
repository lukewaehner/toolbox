//! Network Tools Module
//!
//! This module provides network diagnostic utilities including:
//! - Ping functionality for network connectivity testing
//! - Download speed testing with multiple server options
//!
//! # Examples
//!
//! ```no_run
//! use network_tools::{ping, efficient_speed_test};
//!
//! // Ping a host
//! let result = ping("8.8.8.8").expect("Ping failed");
//! println!("Average RTT: {} ms", result.round_trip_avg);
//!
//! // Test download speed
//! let speed = efficient_speed_test().expect("Speed test failed");
//! println!("Download speed: {} Mbps", speed.speed / 1_000_000.0);
//! ```

use serde::{Deserialize, Serialize};
use std::cmp::min;
use std::io::Read;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// Results from a ping network test
#[derive(Serialize, Deserialize, Debug)]
pub struct PingResult {
    packets_transmitted: u32,
    packets_received: u32,
    packet_loss: f32,
    time: u32,
    round_trip_min: f32,
    round_trip_avg: f32,
    round_trip_max: f32,
    round_trip_mdev: f32,
}

/// Performs a ping test to a specified address
///
/// Sends 4 ICMP echo request packets to the target address and returns statistics.
///
/// # Arguments
///
/// * `address` - The IP address or hostname to ping
///
/// # Returns
///
/// Returns a `PingResult` containing packet transmission statistics and RTT measurements.
///
/// # Errors
///
/// Returns an error if:
/// - The ping command fails to execute
/// - The output cannot be parsed
/// - The target is unreachable
pub fn ping(address: &str) -> Result<PingResult, Box<dyn std::error::Error>> {
    let output = Command::new("ping")
        .arg("-c")
        .arg("4") // Send 4 packets
        .arg(address)
        .stdout(Stdio::piped())
        .output()?;

    if !output.status.success() {
        return Err(format!(
            "Ping failed with status: {}",
            output.status.code().unwrap_or(-1)
        )
        .into());
    }

    let result = String::from_utf8(output.stdout)?;
    println!("Ping command output: {}", result);

    let parsed_result = parse_ping_output(&result)?;
    Ok(parsed_result)
}

/// Parses the output from the ping command
///
/// Extracts packet statistics and RTT measurements from ping command output
/// using regular expressions.
///
/// # Arguments
///
/// * `output` - The stdout from the ping command
///
/// # Returns
///
/// Returns a `PingResult` with parsed statistics.
///
/// # Errors
///
/// Returns an error if the output format doesn't match expected patterns.
fn parse_ping_output(output: &str) -> Result<PingResult, Box<dyn std::error::Error>> {
    use regex::Regex;

    let packets_regex = Regex::new(
        r"(?P<transmitted>\d+) packets transmitted, (?P<received>\d+) (?:packets )?received, (?P<loss>\d+\.?\d*)% packet loss(?:, time (?P<time>\d+)ms)?",
    )?;
    let rtt_regex = Regex::new(
        r"(?:rtt|round-trip) min/avg/max/(?:mdev|stddev) = (?P<min>\d+\.\d+)/(?P<avg>\d+\.\d+)/(?P<max>\d+\.\d+)/(?P<mdev>\d+\.\d+) ms",
    )?;

    let packets_caps = packets_regex
        .captures(output)
        .ok_or("Failed to parse packet stats")?;
    let rtt_caps = rtt_regex
        .captures(output)
        .ok_or("Failed to parse RTT stats")?;

    let packets_transmitted = packets_caps["transmitted"].parse()?;
    let packets_received = packets_caps["received"].parse()?;
    let packet_loss = packets_caps["loss"].parse()?;
    let time = match packets_caps.name("time") {
        Some(m) => m.as_str().parse()?,
        None => 0,
    };

    let round_trip_min = rtt_caps["min"].parse()?;
    let round_trip_avg = rtt_caps["avg"].parse()?;
    let round_trip_max = rtt_caps["max"].parse()?;
    let round_trip_mdev = rtt_caps["mdev"].parse()?;

    Ok(PingResult {
        packets_transmitted,
        packets_received,
        packet_loss,
        time,
        round_trip_min,
        round_trip_avg,
        round_trip_max,
        round_trip_mdev,
    })
}

// Improved SpeedTestResult with additional fields for better reporting
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SpeedTestResult {
    /// Download speed in bits per second
    pub download_speed_bps: f64,
    /// Duration of the test in seconds
    pub duration_secs: f64,
    /// Total bytes downloaded
    pub file_size_bytes: u64,
    /// Status message (for error reporting)
    pub status_message: String,
    /// Test type (single file, multi-file, etc.)
    pub test_type: String,
}

impl SpeedTestResult {
    // Helper to create a status update
    pub fn status(message: &str) -> Self {
        Self {
            download_speed_bps: 0.0,
            duration_secs: 0.0,
            file_size_bytes: 0,
            status_message: message.to_string(),
            test_type: "status".to_string(),
        }
    }

    // Helper to create an error result
    pub fn error(message: &str) -> Self {
        Self {
            download_speed_bps: 0.0,
            duration_secs: 0.0,
            file_size_bytes: 0,
            status_message: format!("Error: {}", message),
            test_type: "error".to_string(),
        }
    }
}

/// Perform a download speed test using an efficient streaming approach
pub fn measure_speed() -> Result<SpeedTestResult, Box<dyn std::error::Error>> {
    // A list of test URLs to try, in order of preference
    let test_urls = [
        "http://speedtest-ny.turnkeyinternet.net/100mb.bin",
        "http://speedtest.tele2.net/50MB.zip",
        "http://speedtest.belwue.net/random-50M",
        "http://ipv4.download.thinkbroadband.com/50MB.zip",
    ];

    // Try each URL until one works
    for url in &test_urls {
        match test_download_speed(url) {
            Ok(result) => return Ok(result),
            Err(e) => eprintln!("Failed to test with {}: {}", url, e),
        }
    }

    // If all URLs fail, return an error
    Err("All speed test servers failed. Check your internet connection.".into())
}

// The core download speed test function
fn test_download_speed(url: &str) -> Result<SpeedTestResult, Box<dyn std::error::Error>> {
    println!("Testing download speed with URL: {}", url);

    // Create a client with appropriate timeouts
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(60))
        .connect_timeout(Duration::from_secs(10))
        .build()?;

    // Start timing
    let _start = Instant::now();

    // Make the request
    let mut response = client.get(url).send()?;

    // Check if the request was successful
    if !response.status().is_success() {
        return Err(format!("Server returned status code: {}", response.status()).into());
    }

    // Get the content length if available
    let content_length = response
        .headers()
        .get(reqwest::header::CONTENT_LENGTH)
        .and_then(|cl| cl.to_str().ok())
        .and_then(|cls| cls.parse::<u64>().ok())
        .unwrap_or(0);

    println!("Content length from headers: {} bytes", content_length);

    // For very large files, we might want to limit how much we download
    // But don't set the limit too low to ensure accurate speed measurement
    let download_limit = 20 * 1024 * 1024; // 20 MB
    let mut bytes_to_read = if content_length > 0 && content_length > download_limit {
        download_limit
    } else if content_length > 0 {
        content_length
    } else {
        download_limit // Default if unknown
    };

    // Stream the response body in chunks to avoid loading everything into memory
    let mut total_bytes = 0u64;
    let mut buffer = vec![0u8; 64 * 1024]; // 64 KB chunks

    // Record when the actual data transfer starts
    let transfer_start = Instant::now();

    // Read the response body in chunks
    while bytes_to_read > 0 {
        let bytes_to_get = min(buffer.len() as u64, bytes_to_read) as usize;

        match response.read(&mut buffer[0..bytes_to_get]) {
            Ok(bytes_read) => {
                if bytes_read == 0 {
                    // End of stream
                    break;
                }

                total_bytes += bytes_read as u64;
                bytes_to_read = bytes_to_read.saturating_sub(bytes_read as u64);

                // Periodically print progress
                if total_bytes % (1024 * 1024) < buffer.len() as u64 {
                    let elapsed = transfer_start.elapsed().as_secs_f64();
                    let mbps = (total_bytes as f64 * 8.0) / (elapsed * 1_000_000.0);
                    println!(
                        "Downloaded {:.2} MB at {:.2} Mbps",
                        total_bytes as f64 / (1024.0 * 1024.0),
                        mbps
                    );
                }
            }
            Err(e) => {
                // Handle read errors
                return Err(format!("Error reading response: {}", e).into());
            }
        }
    }

    // Calculate the total duration from the start of data transfer
    let duration = transfer_start.elapsed().as_secs_f64();

    // Calculate the speed in bits per second
    let speed_bps = if duration > 0.0 {
        (total_bytes as f64 * 8.0) / duration
    } else {
        0.0
    };

    println!("Speed test results:");
    println!("Total bytes: {}", total_bytes);
    println!("Duration: {:.2} seconds", duration);
    println!("Speed: {:.2} Mbps", speed_bps / 1_000_000.0);

    // Return the result
    Ok(SpeedTestResult {
        download_speed_bps: speed_bps,
        duration_secs: duration,
        file_size_bytes: total_bytes,
        status_message: format!(
            "Download complete: {:.2} MB",
            total_bytes as f64 / (1024.0 * 1024.0)
        ),
        test_type: "single_file".to_string(),
    })
}

/// Perform a multi-file download test for more accurate results
pub fn speed_test() -> Result<SpeedTestResult, Box<dyn std::error::Error>> {
    // Start with a single file test
    let single_result = measure_speed()?;

    // If the single file test was fast, try a multi-file test
    if single_result.download_speed_bps > (50.0 * 1_000_000.0) {
        // > 50 Mbps
        // Try the parallel download test
        match parallel_speed_test() {
            Ok(parallel_result) => {
                // Return the higher of the two results
                if parallel_result.download_speed_bps > single_result.download_speed_bps {
                    return Ok(SpeedTestResult {
                        download_speed_bps: parallel_result.download_speed_bps,
                        duration_secs: parallel_result.duration_secs,
                        file_size_bytes: parallel_result.file_size_bytes,
                        status_message: format!(
                            "Multi-file test: {:.2} Mbps",
                            parallel_result.download_speed_bps / 1_000_000.0
                        ),
                        test_type: "multi_file".to_string(),
                    });
                }
            }
            Err(e) => {
                println!("Parallel test failed: {}", e);
                // Continue with single file result
            }
        }
    }

    // Return the single file result if parallel test failed or was slower
    Ok(single_result)
}

/// Perform a parallel download test using multiple simultaneous connections
pub fn parallel_speed_test() -> Result<SpeedTestResult, Box<dyn std::error::Error>> {
    use std::sync::{Arc, Mutex};
    use std::thread;

    // URLs for different test files
    let test_urls = [
        "http://speedtest.tele2.net/10MB.zip",
        "http://ipv4.download.thinkbroadband.com/10MB.zip",
        "http://speedtest.belwue.net/random-10M",
        "http://speedtest-ny.turnkeyinternet.net/10mb.bin",
    ];

    // Number of parallel downloads
    let num_threads = 3.min(test_urls.len());

    // Shared state for results
    let total_bytes = Arc::new(Mutex::new(0u64));

    // Start timing
    let start = Instant::now();

    // Create threads for parallel downloads
    let mut handles = vec![];

    for i in 0..num_threads {
        let url = test_urls[i].to_string();
        let total_bytes_clone = Arc::clone(&total_bytes);

        let handle = thread::spawn(move || {
            match download_chunk(&url) {
                Ok(bytes) => {
                    // Update the total bytes counter
                    let mut total = total_bytes_clone.lock().unwrap();
                    *total += bytes;
                    println!("Thread {} downloaded {} bytes", i, bytes);
                    true
                }
                Err(e) => {
                    eprintln!("Thread {} error: {}", i, e);
                    false
                }
            }
        });

        handles.push(handle);
    }

    // Wait for all threads to complete
    let mut success_count = 0;
    for handle in handles {
        if handle.join().unwrap_or(false) {
            success_count += 1;
        }
    }

    // Calculate duration
    let duration = start.elapsed().as_secs_f64();

    // Get total bytes downloaded
    let total_bytes = *total_bytes.lock().unwrap();

    // Check if any downloads succeeded
    if success_count == 0 || total_bytes == 0 {
        return Err("All parallel downloads failed".into());
    }

    // Calculate speed
    let speed_bps = (total_bytes as f64 * 8.0) / duration;

    println!("Parallel test results:");
    println!("Successful threads: {}/{}", success_count, num_threads);
    println!("Total bytes: {}", total_bytes);
    println!("Duration: {:.2} seconds", duration);
    println!("Speed: {:.2} Mbps", speed_bps / 1_000_000.0);

    Ok(SpeedTestResult {
        download_speed_bps: speed_bps,
        duration_secs: duration,
        file_size_bytes: total_bytes,
        status_message: format!(
            "Multi-file test: {:.2} Mbps from {} sources",
            speed_bps / 1_000_000.0,
            success_count
        ),
        test_type: "parallel".to_string(),
    })
}

// Helper function for parallel downloads
fn download_chunk(url: &str) -> Result<u64, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;

    let mut response = client.get(url).send()?;

    if !response.status().is_success() {
        return Err(format!("Server returned status code: {}", response.status()).into());
    }

    let mut total_bytes = 0u64;
    let mut buffer = vec![0u8; 64 * 1024]; // 64 KB chunks

    loop {
        match response.read(&mut buffer) {
            Ok(bytes_read) => {
                if bytes_read == 0 {
                    break;
                }
                total_bytes += bytes_read as u64;
            }
            Err(e) => {
                return Err(format!("Error reading response: {}", e).into());
            }
        }
    }

    Ok(total_bytes)
}
