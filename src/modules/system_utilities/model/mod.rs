use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use sysinfo::{CpuExt, DiskExt, PidExt, ProcessExt, System, SystemExt};

// Snapshot of system resources for display in the UI
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SystemSnapshot {
    // CPU stats
    pub cpu_usage: f32,            // Global CPU usage percentage
    pub cpu_cores_usage: Vec<f32>, // Per-core CPU usage percentages
    pub cpu_cores_count: usize,
    pub cpu_name: String,

    // Memory stats
    pub memory_used: u64,
    pub memory_total: u64,
    pub memory_usage_percent: f32,
    pub swap_used: u64,
    pub swap_total: u64,
    pub swap_usage_percent: f32,

    // Disk stats
    pub disks: Vec<DiskInfo>,

    // Process stats (top N processes by CPU or memory usage)
    pub top_processes: Vec<ProcessInfo>,

    // Capture timestamp
    pub timestamp: u64,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,
    pub available_space: u64,
    pub total_space: u64,
    pub usage_percent: f32,
    pub is_removable: bool,
    pub filesystem_type: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f32,
    pub memory_usage: u64,
    pub memory_usage_percent: f32,
    pub disk_usage: u64,
    pub start_time: u64,
    pub run_time: u64,
    pub user: String,
}

// Historical data for charting
#[derive(Default, Debug)]
pub struct SystemHistory {
    pub cpu_history: Vec<(u64, f32)>,    // (timestamp, usage)
    pub memory_history: Vec<(u64, f32)>, // (timestamp, usage percent)
    pub history_max_points: usize,
}

impl SystemHistory {
    pub fn new(max_points: usize) -> Self {
        Self {
            cpu_history: Vec::with_capacity(max_points),
            memory_history: Vec::with_capacity(max_points),
            history_max_points: max_points,
        }
    }

    pub fn add_snapshot(&mut self, snapshot: &SystemSnapshot) {
        // Add CPU data point
        self.cpu_history
            .push((snapshot.timestamp, snapshot.cpu_usage));
        if self.cpu_history.len() > self.history_max_points {
            self.cpu_history.remove(0);
        }

        // Add memory data point
        self.memory_history
            .push((snapshot.timestamp, snapshot.memory_usage_percent));
        if self.memory_history.len() > self.history_max_points {
            self.memory_history.remove(0);
        }
    }
}

// Main system monitor that will collect and store data
#[derive(Debug)]
pub struct SystemMonitor {
    system: System,
    snapshot: SystemSnapshot,
    history: SystemHistory,
    refresh_interval: Duration,
    last_update: std::time::Instant,
}

impl SystemMonitor {
    pub fn new(history_points: usize, refresh_interval: Duration) -> Self {
        let mut system = System::new_all();
        system.refresh_all();

        // Create an empty snapshot (will be filled on first refresh)
        let snapshot = SystemSnapshot {
            cpu_usage: 0.0,
            cpu_cores_usage: Vec::new(),
            cpu_cores_count: 0,
            cpu_name: String::new(),
            memory_used: 0,
            memory_total: 0,
            memory_usage_percent: 0.0,
            swap_used: 0,
            swap_total: 0,
            swap_usage_percent: 0.0,
            disks: Vec::new(),
            top_processes: Vec::new(),
            timestamp: 0,
        };

        let history = SystemHistory::new(history_points);

        let mut monitor = Self {
            system,
            snapshot,
            history,
            refresh_interval,
            last_update: std::time::Instant::now(),
        };

        // Do initial refresh
        monitor.refresh();

        monitor
    }

    // Refresh system data if interval has passed
    pub fn refresh_if_needed(&mut self) -> bool {
        if self.last_update.elapsed() >= self.refresh_interval {
            self.refresh();
            true
        } else {
            false
        }
    }

    // Force refresh of system data
    pub fn refresh(&mut self) {
        // Update the timestamp
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Refresh system information
        self.system.refresh_all();

        // CPU usage
        let global_cpu_usage = self.system.global_cpu_info().cpu_usage();
        let mut core_usages = Vec::new();

        for cpu in self.system.cpus() {
            core_usages.push(cpu.cpu_usage());
        }

        // Memory usage
        let memory_used = self.system.used_memory();
        let memory_total = self.system.total_memory();
        let memory_percent = if memory_total > 0 {
            (memory_used as f32 / memory_total as f32) * 100.0
        } else {
            0.0
        };

        let swap_used = self.system.used_swap();
        let swap_total = self.system.total_swap();
        let swap_percent = if swap_total > 0 {
            (swap_used as f32 / swap_total as f32) * 100.0
        } else {
            0.0
        };

        // Disk information
        let mut disks = Vec::new();
        for disk in self.system.disks() {
            let total = disk.total_space();
            let available = disk.available_space();
            let used = total.saturating_sub(available);
            let usage_percent = if total > 0 {
                (used as f32 / total as f32) * 100.0
            } else {
                0.0
            };

            disks.push(DiskInfo {
                name: disk.name().to_string_lossy().to_string(),
                mount_point: disk.mount_point().to_string_lossy().to_string(),
                available_space: available,
                total_space: total,
                usage_percent,
                is_removable: disk.is_removable(),
                filesystem_type: String::from_utf8_lossy(disk.file_system()).to_string(),
            });
        }

        // Process information (top 10 by CPU usage)
        let mut processes = Vec::new();
        for (pid, process) in self.system.processes() {
            processes.push(ProcessInfo {
                pid: pid.as_u32(),
                name: process.name().to_string(),
                cpu_usage: process.cpu_usage(),
                memory_usage: process.memory(),
                memory_usage_percent: if memory_total > 0 {
                    (process.memory() as f32 / memory_total as f32) * 100.0
                } else {
                    0.0
                },
                disk_usage: process.disk_usage().total_read_bytes
                    + process.disk_usage().total_written_bytes,
                start_time: process.start_time(),
                run_time: now.saturating_sub(process.start_time()),
                user: "".to_string(), // Not available in standard sysinfo
            });
        }

        // Sort processes by CPU usage and take top 10
        processes.sort_by(|a, b| {
            b.cpu_usage
                .partial_cmp(&a.cpu_usage)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        processes.truncate(10);

        // Update snapshot
        self.snapshot = SystemSnapshot {
            cpu_usage: global_cpu_usage,
            cpu_cores_usage: core_usages.clone(),
            cpu_cores_count: core_usages.len(),
            cpu_name: self.system.global_cpu_info().name().to_string(),
            memory_used,
            memory_total,
            memory_usage_percent: memory_percent,
            swap_used,
            swap_total,
            swap_usage_percent: swap_percent,
            disks,
            top_processes: processes,
            timestamp: now,
        };

        // Update history
        self.history.add_snapshot(&self.snapshot);

        // Update last refresh time
        self.last_update = std::time::Instant::now();
    }

    // Get the current snapshot
    pub fn snapshot(&self) -> &SystemSnapshot {
        &self.snapshot
    }

    // Get history data
    pub fn history(&self) -> &SystemHistory {
        &self.history
    }

    // Force refresh and get a new snapshot
    pub fn refresh_and_get(&mut self) -> SystemSnapshot {
        self.refresh();
        self.snapshot.clone()
    }

    // Kill a process by PID
    pub fn kill_process(&mut self, pid: u32) -> Result<(), String> {
        // Convert u32 to Pid using from_u32() or as_u32() based on the sysinfo version
        let sys_pid = sysinfo::Pid::from_u32(pid);
        if let Some(process) = self.system.process(sys_pid) {
            if process.kill() {
                Ok(())
            } else {
                Err(format!("Failed to kill process with PID {}", pid))
            }
        } else {
            Err(format!("Process with PID {} not found", pid))
        }
    }
}
