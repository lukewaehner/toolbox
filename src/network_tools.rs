use std::process::{Command, Stdio};
use serde::Serialize;
use serde::Deserialize;

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
