use serde_json::Value;

use crate::collector::brand::brand_from_vendor_or_model;
use crate::hardware::DiskInfo;

use super::util::{first_known, format_bytes, value_array, value_string, value_u64};

pub fn collect(root: &Value) -> Vec<DiskInfo> {
    let physical_disks = value_array(root, "PhysicalDisks");
    if !physical_disks.is_empty() {
        return physical_disks
            .into_iter()
            .map(|disk| {
                let model = first_known(&[
                    value_string(disk, "FriendlyName"),
                    value_string(disk, "Model"),
                ]);

                DiskInfo {
                    device: value_string(disk, "DeviceId"),
                    brand: brand_from_vendor_or_model(
                        Some(&value_string(disk, "Manufacturer")),
                        &model,
                    ),
                    model,
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
                }
            })
            .collect();
    }

    value_array(root, "DiskDrives")
        .into_iter()
        .map(|disk| {
            let model = value_string(disk, "Model");

            DiskInfo {
                device: value_string(disk, "DeviceID"),
                brand: brand_from_vendor_or_model(
                    Some(&value_string(disk, "Manufacturer")),
                    &model,
                ),
                model,
                serial: value_string(disk, "SerialNumber"),
                size: value_u64(disk, "Size")
                    .map(format_bytes)
                    .unwrap_or_else(|| "Unknown".to_string()),
                media_type: value_string(disk, "MediaType"),
                bus: value_string(disk, "InterfaceType"),
                firmware: value_string(disk, "FirmwareRevision"),
                source: "Win32_DiskDrive".to_string(),
                ..DiskInfo::default()
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn physical_disk_brand_falls_back_to_model_prefix_for_ata_manufacturer() {
        let root = json!({
            "PhysicalDisks": [{
                "DeviceId": "0",
                "FriendlyName": "Great Wall GW560 512GB",
                "Manufacturer": "ATA",
                "Model": "Great Wall GW560 512GB",
                "FirmwareVersion": "HP3618B7"
            }]
        });

        let disks = collect(&root);

        assert_eq!(disks[0].brand, "Great Wall");
        assert_eq!(disks[0].firmware, "HP3618B7");
    }

    #[test]
    fn physical_disk_brand_uses_real_manufacturer_first() {
        let root = json!({
            "PhysicalDisks": [{
                "DeviceId": "1",
                "FriendlyName": "Ultra USB 3.0",
                "Manufacturer": "SanDisk",
                "Model": "Ultra USB 3.0"
            }]
        });

        let disks = collect(&root);

        assert_eq!(disks[0].brand, "SanDisk");
    }
}
