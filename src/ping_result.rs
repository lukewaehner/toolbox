#[derive(Serialize, Deserialize, Debug)]
pub struct PingResult {
    packets_transmitted: u32,
    packets_received: u32,
    packet_loss: f32,
    time: Option<u32>, // Changed to Option<u32>
    round_trip_min: f32,
    round_trip_avg: f32,
    round_trip_max: f32,
    round_trip_mdev: f32,
}
