//! System Utilities Module
//!
//! This module provides real-time system monitoring and process management capabilities.
//!
//! # Features
//!
//! - CPU usage monitoring (global and per-core)
//! - Memory and swap usage tracking
//! - Disk space monitoring for all mounted filesystems
//! - Process listing with detailed information
//! - Process management (view, sort, terminate)
//! - Historical data tracking for charting
//!
//! # Example
//!
//! ```no_run
//! use system_utilities::SystemMonitor;
//!
//! let mut monitor = SystemMonitor::new();
//! monitor.refresh();
//! let snapshot = monitor.snapshot();
//!
//! println!("CPU Usage: {:.1}%", snapshot.cpu_usage);
//! println!("Memory: {} / {} bytes", snapshot.memory_used, snapshot.memory_total);
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use sysinfo::{CpuExt, DiskExt, PidExt, ProcessExt, System, SystemExt};

/// Snapshot of system resources at a point in time
///
/// This structure captures a complete snapshot of system state including
/// CPU, memory, disk, and process information for display in the UI.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SystemSnapshot {
    /// Global CPU usage percentage (0-100)
    pub cpu_usage: f32,
    /// Per-core CPU usage percentages
    pub cpu_cores_usage: Vec<f32>,
    /// Number of CPU cores
    pub cpu_cores_count: usize,
    /// CPU model name
    pub cpu_name: String,

    /// Used memory in bytes
    pub memory_used: u64,
    /// Total memory in bytes
    pub memory_total: u64,
    /// Memory usage as percentage (0-100)
    pub memory_usage_percent: f32,
    /// Used swap space in bytes
    pub swap_used: u64,
    /// Total swap space in bytes
    pub swap_total: u64,
    /// Swap usage as percentage (0-100)
    pub swap_usage_percent: f32,

    /// Information about all mounted disks
    pub disks: Vec<DiskInfo>,

    /// Top processes by CPU or memory usage
    pub top_processes: Vec<ProcessInfo>,

    /// Unix timestamp when snapshot was captured
    pub timestamp: u64,
}

/// Information about a disk/filesystem
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct DiskInfo {
    /// Device name
    pub name: String,
    /// Mount point path
    pub mount_point: String,
    /// Available space in bytes
    pub available_space: u64,
    /// Total space in bytes
    pub total_space: u64,
    /// Usage percentage (0-100)
    pub usage_percent: f32,
    /// Whether the disk is removable
    pub is_removable: bool,
    /// Filesystem type (e.g., "ext4", "ntfs")
    pub filesystem_type: String,
}

/// Information about a running process
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ProcessInfo {
    /// Process ID
    pub pid: u32,
    /// Process name
    pub name: String,
    /// CPU usage percentage
    pub cpu_usage: f32,
    /// Memory usage in bytes
    pub memory_usage: u64,
    /// Memory usage as percentage of total memory
    pub memory_usage_percent: f32,
    /// Disk usage in bytes
    pub disk_usage: u64,
    /// Process start time (Unix timestamp)
    pub start_time: u64,
    /// Process runtime in seconds
    pub run_time: u64,
    /// User running the process
    pub user: String,
}

/// Historical system data for charting
///
/// Maintains a rolling window of historical CPU and memory usage
/// for trend visualization.
#[derive(Default, Debug)]
pub struct SystemHistory {
    /// CPU usage history as (timestamp, usage_percent) tuples
    pub cpu_history: Vec<(u64, f32)>,
    /// Memory usage history as (timestamp, usage_percent) tuples
    pub memory_history: Vec<(u64, f32)>,
    /// Maximum number of historical data points to keep
    pub history_max_points: usize,
}

impl SystemHistory {
    /// Creates a new system history tracker
    ///
    /// # Arguments
    ///
    /// * `max_points` - Maximum number of historical data points to keep
    pub fn new(max_points: usize) -> Self {
        Self {
            cpu_history: Vec::with_capacity(max_points),
            memory_history: Vec::with_capacity(max_points),
            history_max_points: max_points,
        }
    }

    /// Adds a snapshot to the historical data
    ///
    /// Maintains a rolling window by removing oldest data points when
    /// the maximum is reached.
    ///
    /// # Arguments
    ///
    /// * `snapshot` - The system snapshot to add to history
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

/// System monitor for real-time resource tracking
///
/// Collects and maintains current and historical system resource information
/// including CPU, memory, disk, and process data.
#[derive(Debug)]
pub struct SystemMonitor {
    /// Underlying sysinfo System instance
    system: System,
    /// Current system state snapshot
    snapshot: SystemSnapshot,
    /// Historical data for trending
    history: SystemHistory,
    /// Minimum time between refreshes
    refresh_interval: Duration,
    /// Time of last refresh
    last_update: std::time::Instant,
}

impl SystemMonitor {
    /// Creates a new system monitor instance
    ///
    /// # Arguments
    ///
    /// * `history_points` - Number of historical data points to maintain
    /// * `refresh_interval` - Minimum duration between data refreshes
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::time::Duration;
    /// use system_utilities::SystemMonitor;
    ///
    /// let monitor = SystemMonitor::new(100, Duration::from_secs(2));
    /// ```
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

    /// Refreshes system data if the refresh interval has elapsed
    ///
    /// # Returns
    ///
    /// Returns `true` if data was refreshed, `false` otherwise
    pub fn refresh_if_needed(&mut self) -> bool {
        if self.last_update.elapsed() >= self.refresh_interval {
            self.refresh();
            true
        } else {
            false
        }
    }

    /// Forces an immediate refresh of system data
    ///
    /// Updates CPU, memory, disk, and process information regardless
    /// of the refresh interval.
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
