use std::collections::HashSet;

use crate::hardware::{unknown, DebugRecord, LogicalDiskInfo};

pub(super) struct ParsedDf {
    pub(super) disks: Vec<LogicalDiskInfo>,
    pub(super) debug: Vec<DebugRecord>,
}

pub(super) fn parse(output: &str, debug_enabled: bool) -> ParsedDf {
    let mut lines = output.lines().filter(|line| !line.trim().is_empty());
    let Some(header) = lines.next() else {
        return ParsedDf {
            disks: Vec::new(),
            debug: Vec::new(),
        };
    };
    let has_type = header
        .split_whitespace()
        .any(|field| field.eq_ignore_ascii_case("type"));

    let mut disks = Vec::new();
    let mut debug = Vec::new();
    let mut seen = HashSet::new();

    for line in lines {
        let fields = line.split_whitespace().collect::<Vec<_>>();
        let Some(row) = parse_row(&fields, has_type) else {
            continue;
        };

        if !seen.insert((row.device.clone(), row.mount_point.clone())) {
            if debug_enabled {
                debug.push(filtered_record(&row, "duplicate device and mount"));
            }
            continue;
        }

        if let Some(reason) = filter_reason(&row) {
            if debug_enabled {
                debug.push(filtered_record(&row, reason));
            }
            continue;
        }

        disks.push(row);
    }

    ParsedDf { disks, debug }
}

fn parse_row(fields: &[&str], has_type: bool) -> Option<LogicalDiskInfo> {
    let (file_system, total_index, used_index, available_index, percent_index, mount_index) =
        if has_type {
            if fields.len() < 7 {
                return None;
            }
            (fields[1].to_string(), 2, 3, 4, 5, 6)
        } else {
            if fields.len() < 6 {
                return None;
            }
            (unknown(), 1, 2, 3, 4, 5)
        };

    let total_kib = fields.get(total_index)?.parse::<u64>().ok()?;
    let used_kib = fields.get(used_index)?.parse::<u64>().ok()?;
    let available_kib = fields.get(available_index)?.parse::<u64>().ok()?;
    let used_percent = fields
        .get(percent_index)?
        .trim_end_matches('%')
        .parse::<f64>()
        .ok()
        .unwrap_or_else(|| {
            if total_kib == 0 {
                0.0
            } else {
                used_kib as f64 / total_kib as f64 * 100.0
            }
        });

    Some(LogicalDiskInfo {
        device: fields[0].to_string(),
        mount_point: fields[mount_index..].join(" "),
        file_system,
        total: crate::telemetry::format_bytes(total_kib.saturating_mul(1024)),
        used: crate::telemetry::format_bytes(used_kib.saturating_mul(1024)),
        available: crate::telemetry::format_bytes(available_kib.saturating_mul(1024)),
        used_percent,
        source: "df".to_string(),
        warnings: Vec::new(),
    })
}

fn filter_reason(row: &LogicalDiskInfo) -> Option<&'static str> {
    let device = row.device.to_ascii_lowercase();
    let file_system = row.file_system.to_ascii_lowercase();
    let mount = row.mount_point.to_ascii_lowercase();
    let pseudo = [
        "tmpfs",
        "devtmpfs",
        "proc",
        "sysfs",
        "cgroup",
        "cgroup2",
        "devpts",
        "debugfs",
        "tracefs",
        "binder",
        "binderfs",
        "pstore",
        "securityfs",
        "configfs",
        "fusectl",
        "bpf",
        "ramfs",
        "overlay",
    ];

    if pseudo.contains(&device.as_str()) || pseudo.contains(&file_system.as_str()) {
        return Some("pseudo filesystem");
    }
    if mount == "/dev"
        || mount.starts_with("/dev/")
        || mount == "/proc"
        || mount.starts_with("/proc/")
        || mount == "/sys"
        || mount.starts_with("/sys/")
    {
        return Some("kernel or device mount");
    }
    if device.contains("/loop") {
        return Some("loop-backed system mount");
    }

    None
}

fn filtered_record(row: &LogicalDiskInfo, reason: &str) -> DebugRecord {
    DebugRecord::new(row.mount_point.clone(), "df-filter")
        .field("device", row.device.clone())
        .field("filesystem", row.file_system.clone())
        .field("reason", reason)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_coreutils_df_with_filesystem_type() {
        let output = "Filesystem Type 1024-blocks Used Available Capacity Mounted on\n/dev/block/dm-0 ext4 1048576 262144 786432 25% /data\n";
        let parsed = parse(output, false);

        assert_eq!(parsed.disks.len(), 1);
        assert_eq!(parsed.disks[0].device, "/dev/block/dm-0");
        assert_eq!(parsed.disks[0].file_system, "ext4");
        assert_eq!(parsed.disks[0].mount_point, "/data");
        assert_eq!(parsed.disks[0].total, "1.00 GiB");
        assert_eq!(parsed.disks[0].used_percent, 25.0);
    }

    #[test]
    fn parses_toybox_df_without_filesystem_type() {
        let output = "Filesystem 1K-blocks Used Available Use% Mounted on\n/dev/fuse 2097152 1048576 1048576 50% /storage/emulated\n";
        let parsed = parse(output, false);

        assert_eq!(parsed.disks.len(), 1);
        assert_eq!(parsed.disks[0].mount_point, "/storage/emulated");
        assert_eq!(parsed.disks[0].file_system, unknown());
        assert_eq!(parsed.disks[0].used, "1.00 GiB");
    }

    #[test]
    fn filters_pseudo_filesystems_from_default_rows() {
        let output = "Filesystem Type 1024-blocks Used Available Capacity Mounted on\ntmpfs tmpfs 1024 0 1024 0% /dev\n/dev/block/dm-0 ext4 2048 1024 1024 50% /data\n";
        let parsed = parse(output, false);

        assert_eq!(parsed.disks.len(), 1);
        assert_eq!(parsed.disks[0].mount_point, "/data");
        assert!(parsed.debug.is_empty());
    }

    #[test]
    fn records_filtered_mounts_when_debug_is_enabled() {
        let output = "Filesystem Type 1024-blocks Used Available Capacity Mounted on\ntmpfs tmpfs 1024 0 1024 0% /dev\n";
        let parsed = parse(output, true);

        assert!(parsed.disks.is_empty());
        assert_eq!(parsed.debug.len(), 1);
        assert_eq!(parsed.debug[0].source, "df-filter");
        assert_eq!(parsed.debug[0].fields["reason"], "pseudo filesystem");
    }
}
