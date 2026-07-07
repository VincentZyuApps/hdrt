use std::fs;
use std::path::{Path, PathBuf};

use crate::hardware::{unknown, DiskInfo};

use super::brand::brand_from_vendor_or_model;
use super::command::{format_bytes, non_empty_or_unknown};

pub(super) fn collect() -> Vec<DiskInfo> {
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
    let vendor = read_sysfs(&path, "device/vendor");
    let bus = infer_bus(&path);
    let rota = read_sysfs(&path, "queue/rotational");

    Some(DiskInfo {
        device,
        model: model.clone(),
        brand: brand_from_vendor_or_model(Some(vendor.as_str()), &model),
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
        .map(|sectors| format_bytes(sectors.saturating_mul(512)))
        .unwrap_or_else(unknown)
}

fn infer_bus(path: &Path) -> String {
    let canonical = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let text = canonical.to_string_lossy().to_ascii_lowercase();

    if text.contains("/nvme") {
        "nvme".to_string()
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

    match rota {
        "0" => "SSD".to_string(),
        "1" => "HDD".to_string(),
        _ => unknown(),
    }
}

fn first_known(values: &[String]) -> String {
    values
        .iter()
        .find(|value| !crate::hardware::is_unknown(value))
        .cloned()
        .unwrap_or_else(unknown)
}
