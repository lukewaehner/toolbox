#[derive(serde::Deserialize, Debug)]
pub struct PingResult {
    pub packets_transmitted: u32,
    pub packets_received: u32,
    pub packet_loss: f32,
    pub time: Option<u32>,
    pub round_trip_min: f32,
    pub round_trip_avg: f32,
    pub round_trip_max: f32,
    pub round_trip_mdev: f32,
}
