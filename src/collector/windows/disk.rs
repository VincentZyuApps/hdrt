use serde_json::Value;

use crate::hardware::DiskInfo;

use super::util::{clean_manufacturer, format_bytes, value_array, value_string, value_u64};

pub fn collect(root: &Value) -> Vec<DiskInfo> {
    let physical_disks = value_array(root, "PhysicalDisks");
    if !physical_disks.is_empty() {
        return physical_disks
            .into_iter()
            .map(|disk| DiskInfo {
                device: value_string(disk, "DeviceId"),
                model: value_string(disk, "FriendlyName"),
                serial: value_string(disk, "SerialNumber"),
                size: value_u64(disk, "Size")
                    .map(format_bytes)
                    .unwrap_or_else(|| "Unknown".to_string()),
                media_type: value_string(disk, "MediaType"),
                bus: value_string(disk, "BusType"),
                firmware: value_string(disk, "FirmwareVersion"),
                health: value_string(disk, "HealthStatus"),
                source: "Get-PhysicalDisk".to_string(),
                ..DiskInfo::default()
            })
            .collect();
    }

    value_array(root, "DiskDrives")
        .into_iter()
        .map(|disk| DiskInfo {
            device: value_string(disk, "DeviceID"),
            model: value_string(disk, "Model"),
            brand: clean_manufacturer(&value_string(disk, "Manufacturer")),
            serial: value_string(disk, "SerialNumber"),
            size: value_u64(disk, "Size")
                .map(format_bytes)
                .unwrap_or_else(|| "Unknown".to_string()),
            media_type: value_string(disk, "MediaType"),
            bus: value_string(disk, "InterfaceType"),
            firmware: value_string(disk, "FirmwareRevision"),
            source: "Win32_DiskDrive".to_string(),
            ..DiskInfo::default()
        })
        .collect()
}
