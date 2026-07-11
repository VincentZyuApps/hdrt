use sysinfo::Disks;

use crate::hardware::LogicalDiskInfo;

pub(super) fn collect() -> Vec<LogicalDiskInfo> {
    let disks = Disks::new_with_refreshed_list();
    disks
        .list()
        .iter()
        .map(|disk| {
            let total = disk.total_space();
            let available = disk.available_space();
            let used = total.saturating_sub(available);

            LogicalDiskInfo {
                device: disk.name().to_string_lossy().into_owned(),
                mount_point: disk.mount_point().display().to_string(),
                file_system: disk.file_system().to_string_lossy().into_owned(),
                total: crate::telemetry::format_bytes(total),
                used: crate::telemetry::format_bytes(used),
                available: crate::telemetry::format_bytes(available),
                used_percent: if total == 0 {
                    0.0
                } else {
                    used as f64 / total as f64 * 100.0
                },
                source: "sysinfo".to_string(),
                warnings: Vec::new(),
            }
        })
        .collect()
}
