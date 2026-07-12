use sysinfo::{Disks, System};

use crate::hardware::{is_unknown, LogicalDiskInfo};

mod disk_io;

use disk_io::{DiskIoCounter, DiskIoSampler};

pub const DEFAULT_INTERVAL_MS: u64 = 1_000;
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
    pub io_available: bool,
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

    pub fn sample(&mut self, inventory: &[LogicalDiskInfo]) -> TelemetrySnapshot {
        self.system.refresh_cpu_usage();
        self.system.refresh_memory();
        self.disks.refresh();
        let disk_io = self.disk_io_sampler.sample();
        let live_disks = self
            .disks
            .list()
            .iter()
            .map(|disk| {
                let total = disk.total_space();
                let available = disk.available_space();
                let used = total.saturating_sub(available);
                LiveDisk {
                    name: disk.name().to_string_lossy().into_owned(),
                    mount_point: disk.mount_point().display().to_string(),
                    file_system: disk.file_system().to_string_lossy().into_owned(),
                    total_bytes: total,
                    available_bytes: available,
                    used_bytes: used,
                    used_percent: percent(used, total),
                }
            })
            .collect::<Vec<_>>();

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
            disks: merge_disk_inventory(inventory, &live_disks, &disk_io),
        }
    }
}

impl Default for TelemetrySampler {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Default)]
struct LiveDisk {
    name: String,
    mount_point: String,
    file_system: String,
    total_bytes: u64,
    available_bytes: u64,
    used_bytes: u64,
    used_percent: f64,
}

fn merge_disk_inventory(
    inventory: &[LogicalDiskInfo],
    live_disks: &[LiveDisk],
    disk_io: &[DiskIoCounter],
) -> Vec<DiskTelemetry> {
    if inventory.is_empty() {
        return live_disks
            .iter()
            .map(|disk| telemetry_from_live(disk, disk_io))
            .collect();
    }

    inventory
        .iter()
        .map(|item| {
            let live = live_disks.iter().find(|disk| disk_matches(item, disk));
            telemetry_from_inventory(item, live, disk_io)
        })
        .collect()
}

fn telemetry_from_live(disk: &LiveDisk, disk_io: &[DiskIoCounter]) -> DiskTelemetry {
    let io = find_disk_io(disk_io, &disk.mount_point, &disk.name);
    DiskTelemetry {
        name: disk.name.clone(),
        mount_point: disk.mount_point.clone(),
        file_system: disk.file_system.clone(),
        total_bytes: disk.total_bytes,
        available_bytes: disk.available_bytes,
        used_bytes: disk.used_bytes,
        used_percent: disk.used_percent,
        io_available: io.is_some(),
        read_bytes_per_sec: io.map(|item| item.read_bytes_per_sec).unwrap_or_default(),
        write_bytes_per_sec: io.map(|item| item.write_bytes_per_sec).unwrap_or_default(),
        total_read_bytes: io.map(|item| item.total_read_bytes).unwrap_or_default(),
        total_written_bytes: io.map(|item| item.total_written_bytes).unwrap_or_default(),
    }
}

fn telemetry_from_inventory(
    item: &LogicalDiskInfo,
    live: Option<&LiveDisk>,
    disk_io: &[DiskIoCounter],
) -> DiskTelemetry {
    let name = preferred_text(&item.device, live.map(|disk| disk.name.as_str()));
    let mount_point = preferred_text(
        &item.mount_point,
        live.map(|disk| disk.mount_point.as_str()),
    );
    let file_system = preferred_text(
        &item.file_system,
        live.map(|disk| disk.file_system.as_str()),
    );
    let io = find_disk_io(disk_io, &mount_point, &name);

    DiskTelemetry {
        name,
        mount_point,
        file_system,
        total_bytes: live
            .map(|disk| disk.total_bytes)
            .or_else(|| parse_storage_bytes(&item.total))
            .unwrap_or_default(),
        available_bytes: live
            .map(|disk| disk.available_bytes)
            .or_else(|| parse_storage_bytes(&item.available))
            .unwrap_or_default(),
        used_bytes: live
            .map(|disk| disk.used_bytes)
            .or_else(|| parse_storage_bytes(&item.used))
            .unwrap_or_default(),
        used_percent: live
            .map(|disk| disk.used_percent)
            .unwrap_or(item.used_percent),
        io_available: io.is_some(),
        read_bytes_per_sec: io.map(|value| value.read_bytes_per_sec).unwrap_or_default(),
        write_bytes_per_sec: io
            .map(|value| value.write_bytes_per_sec)
            .unwrap_or_default(),
        total_read_bytes: io.map(|value| value.total_read_bytes).unwrap_or_default(),
        total_written_bytes: io
            .map(|value| value.total_written_bytes)
            .unwrap_or_default(),
    }
}

fn disk_matches(item: &LogicalDiskInfo, live: &LiveDisk) -> bool {
    same_identity(&item.mount_point, &live.mount_point) || same_identity(&item.device, &live.name)
}

fn same_identity(left: &str, right: &str) -> bool {
    if left.trim().is_empty() || right.trim().is_empty() || is_unknown(left) || is_unknown(right) {
        return false;
    }
    normalize_disk_key(left).eq_ignore_ascii_case(&normalize_disk_key(right))
}

fn preferred_text(primary: &str, fallback: Option<&str>) -> String {
    if !primary.trim().is_empty() && !is_unknown(primary) {
        primary.to_string()
    } else {
        fallback.unwrap_or(primary).to_string()
    }
}

fn parse_storage_bytes(value: &str) -> Option<u64> {
    let value = value.trim();
    if value.is_empty() || is_unknown(value) {
        return None;
    }

    let number_len = value
        .chars()
        .take_while(|ch| ch.is_ascii_digit() || *ch == '.')
        .map(char::len_utf8)
        .sum::<usize>();
    let amount = value.get(..number_len)?.parse::<f64>().ok()?;
    let unit = value.get(number_len..)?.trim().to_ascii_lowercase();
    let multiplier = if unit.starts_with("tib") {
        1024.0_f64.powi(4)
    } else if unit.starts_with("gib") {
        1024.0_f64.powi(3)
    } else if unit.starts_with("mib") {
        1024.0_f64.powi(2)
    } else if unit.starts_with("kib") {
        1024.0
    } else {
        1.0
    };

    Some((amount * multiplier).round().clamp(0.0, u64::MAX as f64) as u64)
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

fn find_disk_io<'a>(
    counters: &'a [DiskIoCounter],
    mount_point: &str,
    disk_name: &str,
) -> Option<&'a DiskIoCounter> {
    let mount_key = normalize_disk_key(mount_point);
    let name_key = normalize_disk_key(disk_name);
    counters.iter().find(|counter| {
        let counter_key = normalize_disk_key(&counter.name);
        (!mount_key.is_empty() && counter_key.eq_ignore_ascii_case(&mount_key))
            || (!name_key.is_empty() && counter_key.eq_ignore_ascii_case(&name_key))
    })
}

fn normalize_disk_key(value: &str) -> String {
    let value = value.trim();
    if value == "/" || value == "\\" {
        return value.to_string();
    }
    value.trim_end_matches(['\\', '/']).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inventory_disk() -> LogicalDiskInfo {
        LogicalDiskInfo {
            device: "/dev/block/dm-8".to_string(),
            mount_point: "/data".to_string(),
            file_system: "ext4".to_string(),
            total: "2.00 GiB".to_string(),
            used: "1.00 GiB".to_string(),
            available: "1.00 GiB".to_string(),
            used_percent: 50.0,
            source: "df".to_string(),
            warnings: Vec::new(),
        }
    }

    #[test]
    fn inventory_identity_is_kept_while_live_capacity_is_refreshed() {
        let inventory = vec![inventory_disk()];
        let live = vec![LiveDisk {
            name: "/dev/block/dm-8".to_string(),
            mount_point: "/data/".to_string(),
            file_system: "f2fs".to_string(),
            total_bytes: 3_000,
            available_bytes: 1_000,
            used_bytes: 2_000,
            used_percent: 66.7,
        }];
        let io = vec![DiskIoCounter {
            name: "/data".to_string(),
            read_bytes_per_sec: 512.0,
            write_bytes_per_sec: 256.0,
            total_read_bytes: 4_096,
            total_written_bytes: 2_048,
        }];

        let merged = merge_disk_inventory(&inventory, &live, &io);

        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].name, "/dev/block/dm-8");
        assert_eq!(merged[0].mount_point, "/data");
        assert_eq!(merged[0].file_system, "ext4");
        assert_eq!(merged[0].used_bytes, 2_000);
        assert!(merged[0].io_available);
        assert_eq!(merged[0].read_bytes_per_sec, 512.0);
    }

    #[test]
    fn inventory_rows_survive_when_sysinfo_cannot_see_the_mount() {
        let merged = merge_disk_inventory(&[inventory_disk()], &[], &[]);

        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].total_bytes, 2 * 1024 * 1024 * 1024);
        assert_eq!(merged[0].used_bytes, 1024 * 1024 * 1024);
        assert!(!merged[0].io_available);
    }

    #[test]
    fn root_mount_key_is_not_trimmed_to_empty() {
        assert_eq!(normalize_disk_key("/"), "/");
        assert_eq!(normalize_disk_key("C:\\"), "C:");
    }

    #[test]
    fn parses_formatted_storage_sizes() {
        assert_eq!(parse_storage_bytes("1.50 KiB"), Some(1_536));
        assert_eq!(parse_storage_bytes("2 B"), Some(2));
        assert_eq!(parse_storage_bytes(crate::hardware::UNKNOWN), None);
    }
}
