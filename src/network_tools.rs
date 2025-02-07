use serde::{Deserialize, Serialize};
use std::process::{Command, Stdio};
use std::time::Instant;

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

pub fn ping(address: &str) -> Result<PingResult, Box<dyn std::error::Error>> {
    // println!("Executing ping command for address: {}", address); // Debug print

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
    println!("Ping command output: {}", result); // Debug print

    let parsed_result = parse_ping_output(&result)?;
    Ok(parsed_result)
}

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

#[derive(Serialize, Deserialize, Debug)]
pub struct SpeedTestResult {
    /// Instantaneous download speed in bits per second.
    pub download_speed_bps: f64,
    /// Duration of this measurement (in seconds).
    pub duration_secs: f64,
    /// Total number of bytes downloaded.
    pub file_size_bytes: u64,
}

/// Measures the download speed by downloading a test file (or a part of one).
/// Adjust the URL and logic as needed.
pub fn measure_speed() -> Result<SpeedTestResult, Box<dyn std::error::Error>> {
    // For example, download a 1MB file.
    let test_url = "http://ipv4.download.thinkbroadband.com/1MB.zip";
    let start = Instant::now();
    let response = reqwest::blocking::get(test_url)?;
    let bytes = response.bytes()?;
    let file_size = bytes.len() as u64;
    let duration = start.elapsed();
    let seconds = duration.as_secs_f64();
    if seconds == 0.0 {
        return Err("Download completed too quickly to measure speed".into());
    }
    let speed_bps = (file_size as f64 * 8.0) / seconds;
    Ok(SpeedTestResult {
        download_speed_bps: speed_bps,
        duration_secs: seconds,
        file_size_bytes: file_size,
    })
}

/// Performs a download speed test by fetching a file from a known URL and timing the download.
///
/// This function downloads a file (10 MB in this example) and calculates the speed based on
/// the file size and elapsed time. Adjust the URL or logic as needed.
pub fn speed_test() -> Result<SpeedTestResult, Box<dyn std::error::Error>> {
    // URL for a test file. You can change this to any file with a known size.
    let test_url = "http://ipv4.download.thinkbroadband.com/10MB.zip";

    // Start timing the download.
    let start = Instant::now();

    // Perform a blocking HTTP GET request.
    let response = reqwest::blocking::get(test_url)?;

    // Read the response bytes.
    let bytes = response.bytes()?;
    let file_size = bytes.len() as u64;

    // Measure the elapsed time.
    let duration = start.elapsed();
    let seconds = duration.as_secs_f64();

    // Avoid division by zero.
    if seconds == 0.0 {
        return Err("Download completed too quickly to measure speed.".into());
    }

    // Calculate the download speed in bits per second.
    let speed_bps = (file_size as f64 * 8.0) / seconds;

    Ok(SpeedTestResult {
        download_speed_bps: speed_bps,
        duration_secs: seconds,
        file_size_bytes: file_size,
    })
}
