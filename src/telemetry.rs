use sysinfo::{Disks, System};

#[cfg(windows)]
use serde::Deserialize;
#[cfg(windows)]
use wmi::WMIConnection;

pub const DEFAULT_INTERVAL_MS: u64 = 2_000;
pub const MIN_INTERVAL_MS: u64 = 250;
pub const HISTORY_LIMIT: usize = 240;

#[derive(Debug, Clone, Default)]
pub struct TelemetrySnapshot {
    pub cpu_total_percent: f64,
    pub cpu_cores_percent: Vec<f64>,
    pub memory: MemoryTelemetry,
    pub disks: Vec<DiskTelemetry>,
}

#[derive(Debug, Clone, Default)]
pub struct MemoryTelemetry {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub swap_total_bytes: u64,
    pub swap_used_bytes: u64,
    pub used_percent: f64,
    pub swap_used_percent: f64,
}

#[derive(Debug, Clone, Default)]
pub struct DiskTelemetry {
    pub name: String,
    pub mount_point: String,
    pub file_system: String,
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub used_bytes: u64,
    pub used_percent: f64,
    pub read_bytes_per_sec: f64,
    pub write_bytes_per_sec: f64,
    pub total_read_bytes: u64,
    pub total_written_bytes: u64,
}

pub struct TelemetrySampler {
    system: System,
    disks: Disks,
    disk_io_sampler: DiskIoSampler,
}

impl TelemetrySampler {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_cpu_all();
        system.refresh_memory();

        let mut disks = Disks::new_with_refreshed_list();
        disks.refresh();

        Self {
            system,
            disks,
            disk_io_sampler: DiskIoSampler::new(),
        }
    }

    pub fn sample(&mut self) -> TelemetrySnapshot {
        self.system.refresh_cpu_usage();
        self.system.refresh_memory();
        self.disks.refresh();
        let disk_io = self.disk_io_sampler.sample();

        let total_memory = self.system.total_memory();
        let used_memory = self.system.used_memory();
        let total_swap = self.system.total_swap();
        let used_swap = self.system.used_swap();

        TelemetrySnapshot {
            cpu_total_percent: clamp_percent(self.system.global_cpu_usage() as f64),
            cpu_cores_percent: self
                .system
                .cpus()
                .iter()
                .map(|cpu| clamp_percent(cpu.cpu_usage() as f64))
                .collect(),
            memory: MemoryTelemetry {
                total_bytes: total_memory,
                used_bytes: used_memory,
                available_bytes: self.system.available_memory(),
                swap_total_bytes: total_swap,
                swap_used_bytes: used_swap,
                used_percent: percent(used_memory, total_memory),
                swap_used_percent: percent(used_swap, total_swap),
            },
            disks: self
                .disks
                .list()
                .iter()
                .map(|disk| {
                    let total = disk.total_space();
                    let available = disk.available_space();
                    let used = total.saturating_sub(available);
                    let mount_point = disk.mount_point().display().to_string();
                    let name = disk.name().to_string_lossy().into_owned();
                    let io = find_disk_io(&disk_io, &mount_point, &name);
                    DiskTelemetry {
                        name,
                        mount_point,
                        file_system: disk.file_system().to_string_lossy().into_owned(),
                        total_bytes: total,
                        available_bytes: available,
                        used_bytes: used,
                        used_percent: percent(used, total),
                        read_bytes_per_sec: io
                            .as_ref()
                            .map(|io| io.read_bytes_per_sec)
                            .unwrap_or_default(),
                        write_bytes_per_sec: io
                            .as_ref()
                            .map(|io| io.write_bytes_per_sec)
                            .unwrap_or_default(),
                        total_read_bytes: 0,
                        total_written_bytes: 0,
                    }
                })
                .collect(),
        }
    }
}

impl Default for TelemetrySampler {
    fn default() -> Self {
        Self::new()
    }
}

pub fn normalized_interval_ms(value: u64) -> u64 {
    value.max(MIN_INTERVAL_MS)
}

pub fn format_bytes(bytes: u64) -> String {
    let value = bytes as f64;
    if value >= 1024.0 * 1024.0 * 1024.0 * 1024.0 {
        format!("{:.2} TiB", value / 1024.0 / 1024.0 / 1024.0 / 1024.0)
    } else if value >= 1024.0 * 1024.0 * 1024.0 {
        format!("{:.2} GiB", value / 1024.0 / 1024.0 / 1024.0)
    } else if value >= 1024.0 * 1024.0 {
        format!("{:.2} MiB", value / 1024.0 / 1024.0)
    } else if value >= 1024.0 {
        format!("{:.2} KiB", value / 1024.0)
    } else {
        format!("{bytes} B")
    }
}

pub fn format_rate(bytes_per_sec: f64) -> String {
    if bytes_per_sec >= 1024.0 * 1024.0 * 1024.0 {
        format!("{:.2} GiB/s", bytes_per_sec / 1024.0 / 1024.0 / 1024.0)
    } else if bytes_per_sec >= 1024.0 * 1024.0 {
        format!("{:.2} MiB/s", bytes_per_sec / 1024.0 / 1024.0)
    } else if bytes_per_sec >= 1024.0 {
        format!("{:.2} KiB/s", bytes_per_sec / 1024.0)
    } else {
        format!("{bytes_per_sec:.0} B/s")
    }
}

pub fn format_percent(value: f64) -> String {
    format!("{:.1}%", clamp_percent(value))
}

fn percent(used: u64, total: u64) -> f64 {
    if total == 0 {
        0.0
    } else {
        clamp_percent(used as f64 / total as f64 * 100.0)
    }
}

fn clamp_percent(value: f64) -> f64 {
    value.clamp(0.0, 100.0)
}

#[derive(Debug, Clone, Default)]
struct DiskIoCounter {
    name: String,
    read_bytes_per_sec: f64,
    write_bytes_per_sec: f64,
}

fn find_disk_io<'a>(
    counters: &'a [DiskIoCounter],
    mount_point: &str,
    disk_name: &str,
) -> Option<&'a DiskIoCounter> {
    let key = disk_io_key(mount_point, disk_name);
    counters
        .iter()
        .find(|counter| counter.name.eq_ignore_ascii_case(&key))
}

fn disk_io_key(mount_point: &str, disk_name: &str) -> String {
    let mount = mount_point
        .trim()
        .trim_end_matches(|ch| ch == '\\' || ch == '/');
    if !mount.is_empty() {
        return mount.to_string();
    }

    disk_name
        .trim()
        .trim_end_matches(|ch| ch == '\\' || ch == '/')
        .to_string()
}

#[cfg(windows)]
struct DiskIoSampler {
    conn: Option<WMIConnection>,
}

#[cfg(windows)]
impl DiskIoSampler {
    fn new() -> Self {
        Self {
            conn: WMIConnection::with_namespace_path("ROOT\\CIMV2").ok(),
        }
    }

    fn sample(&mut self) -> Vec<DiskIoCounter> {
        let Some(conn) = &self.conn else {
            return Vec::new();
        };

        let query = "SELECT Name, DiskReadBytesPersec, DiskWriteBytesPersec \
            FROM Win32_PerfFormattedData_PerfDisk_LogicalDisk";
        conn.raw_query::<Win32LogicalDiskPerf>(query)
            .map(|rows| {
                rows.into_iter()
                    .filter(|row| row.name.as_deref() != Some("_Total"))
                    .map(|row| DiskIoCounter {
                        name: row.name.unwrap_or_default(),
                        read_bytes_per_sec: row.disk_read_bytes_per_sec.unwrap_or_default() as f64,
                        write_bytes_per_sec: row.disk_write_bytes_per_sec.unwrap_or_default()
                            as f64,
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}

#[cfg(windows)]
#[derive(Debug, Deserialize)]
struct Win32LogicalDiskPerf {
    #[serde(rename = "Name")]
    name: Option<String>,
    #[serde(rename = "DiskReadBytesPersec")]
    disk_read_bytes_per_sec: Option<u64>,
    #[serde(rename = "DiskWriteBytesPersec")]
    disk_write_bytes_per_sec: Option<u64>,
}

#[cfg(not(windows))]
struct DiskIoSampler;

#[cfg(not(windows))]
impl DiskIoSampler {
    fn new() -> Self {
        Self
    }

    fn sample(&mut self) -> Vec<DiskIoCounter> {
        Vec::new()
    }
}
