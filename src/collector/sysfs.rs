use std::fs;
use std::path::{Path, PathBuf};

use crate::hardware::{is_unknown, unknown, DiskInfo};

pub(super) fn collect_physical_disks() -> Vec<DiskInfo> {
    let Ok(entries) = fs::read_dir("/sys/block") else {
        return Vec::new();
    };

    entries
        .filter_map(Result::ok)
        .filter_map(|entry| disk_from_sysfs(entry.path()))
        .collect()
}

fn disk_from_sysfs(path: PathBuf) -> Option<DiskInfo> {
    let device = path.file_name()?.to_string_lossy().to_string();
    if should_skip_device(&device) {
        return None;
    }

    let model = read_sysfs(&path, "device/model");
    let bus = infer_bus(&path);
    let rota = read_sysfs(&path, "queue/rotational");

    Some(DiskInfo {
        device,
        model,
        serial: read_sysfs(&path, "device/serial"),
        size: read_size(&path),
        media_type: media_type(&rota, &bus),
        bus,
        firmware: first_known(&[
            read_sysfs(&path, "device/firmware_rev"),
            read_sysfs(&path, "device/rev"),
        ]),
        health: unknown(),
        source: "/sys/block".to_string(),
        ..DiskInfo::default()
    })
}

fn should_skip_device(name: &str) -> bool {
    name.starts_with("loop")
        || name.starts_with("ram")
        || name.starts_with("zram")
        || name.starts_with("fd")
        || name.starts_with("dm-")
        || name.starts_with("md")
        || is_mmc_special_device(name)
}

fn is_mmc_special_device(name: &str) -> bool {
    let Some(rest) = name.strip_prefix("mmcblk") else {
        return false;
    };
    let suffix = rest.trim_start_matches(|ch: char| ch.is_ascii_digit());
    suffix.starts_with("boot") || suffix == "rpmb" || suffix.starts_with("gp")
}

fn read_sysfs(base: &Path, relative: &str) -> String {
    fs::read_to_string(base.join(relative))
        .map(|value| non_empty_or_unknown(value.trim()))
        .unwrap_or_else(|_| unknown())
}

fn read_size(path: &Path) -> String {
    fs::read_to_string(path.join("size"))
        .ok()
        .and_then(|value| value.trim().parse::<u64>().ok())
        .map(|sectors| crate::telemetry::format_bytes(sectors.saturating_mul(512)))
        .unwrap_or_else(unknown)
}

fn infer_bus(path: &Path) -> String {
    let canonical = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let text = canonical.to_string_lossy().to_ascii_lowercase();

    if text.contains("/nvme") {
        "nvme".to_string()
    } else if text.contains("ufs") {
        "ufs".to_string()
    } else if text.contains("/usb") {
        "usb".to_string()
    } else if text.contains("/ata") {
        "sata".to_string()
    } else if text.contains("/virtio") {
        "virtio".to_string()
    } else if text.contains("/mmc") {
        "mmc".to_string()
    } else if text.contains("/scsi") {
        "scsi".to_string()
    } else {
        unknown()
    }
}

fn media_type(rota: &str, bus: &str) -> String {
    if bus.eq_ignore_ascii_case("nvme") {
        return "NVMe SSD".to_string();
    }
    if bus.eq_ignore_ascii_case("ufs") || bus.eq_ignore_ascii_case("mmc") {
        return "Flash".to_string();
    }

    match rota {
        "0" => "SSD".to_string(),
        "1" => "HDD".to_string(),
        _ => unknown(),
    }
}

fn first_known(values: &[String]) -> String {
    values
        .iter()
        .find(|value| !is_unknown(value))
        .cloned()
        .unwrap_or_else(unknown)
}

fn non_empty_or_unknown(value: &str) -> String {
    if value.trim().is_empty() {
        unknown()
    } else {
        value.trim().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skips_virtual_block_devices() {
        for name in [
            "loop0",
            "ram0",
            "zram0",
            "dm-0",
            "md0",
            "mmcblk0boot0",
            "mmcblk0boot1",
            "mmcblk0rpmb",
            "mmcblk0gp0",
        ] {
            assert!(should_skip_device(name));
        }
        assert!(!should_skip_device("sda"));
        assert!(!should_skip_device("mmcblk0"));
        assert!(!should_skip_device("mmcblk0p1"));
    }

    #[test]
    fn classifies_common_flash_buses() {
        assert_eq!(media_type("0", "nvme"), "NVMe SSD");
        assert_eq!(media_type("0", "ufs"), "Flash");
        assert_eq!(media_type("0", "mmc"), "Flash");
        assert_eq!(media_type("1", "sata"), "HDD");
    }
}
