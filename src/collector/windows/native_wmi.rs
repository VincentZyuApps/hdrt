use serde::de::DeserializeOwned;
use std::collections::BTreeSet;
use wmi::WMIConnection;

use crate::collector::CollectOptions;
use crate::hardware::{unknown, DebugRecord, DiskInfo, HardwareReport, HdrtWarning};

use super::native_storage::StorageDescriptor;
use super::util::{first_known, format_bytes};
use self::rows::{MsftPhysicalDisk, Win32DiskDrive};

mod inventory;
mod rows;

pub fn collect_report(options: CollectOptions) -> Result<HardwareReport, String> {
    let cimv2 = connect_namespace("ROOT\\CIMV2")?;
    let mut warnings = Vec::new();
    let mut debug = Vec::new();

    let storage_disks = match collect_storage_disks(options.debug, &mut debug) {
        Ok(disks) => disks,
        Err(err) => {
            warnings.push(HdrtWarning::with_hint(
                "windows-native-wmi-storage-unavailable",
                err,
                "Falling back to Win32_DiskDrive for disk inventory.",
            ));
            Vec::new()
        }
    };
    let disk_drives = collect_disk_drive_rows(&cimv2, &mut warnings);
    let descriptors =
        collect_storage_descriptors(&storage_disks, &disk_drives, options.debug, &mut debug);

    let disks = if storage_disks.is_empty() {
        disk_drive_rows_to_disks(&disk_drives, &descriptors, options.debug, &mut debug)
    } else {
        storage_rows_to_disks(
            &storage_disks,
            &disk_drives,
            &descriptors,
            options.debug,
            &mut debug,
        )
    };

    let memory = inventory::collect_memory(&cimv2, &mut warnings);
    let cpu = inventory::collect_cpu(&cimv2, &mut warnings);
    let motherboard = inventory::collect_motherboard(&cimv2, &mut warnings);

    let report = HardwareReport {
        physical_disks: disks,
        logical_disks: Vec::new(),
        memory,
        cpu,
        motherboard,
        warnings,
        debug,
    };

    if report.physical_disks.is_empty()
        && report.memory.is_empty()
        && report.cpu.is_none()
        && report.motherboard.is_none()
    {
        Err("native WMI returned no hardware data".to_string())
    } else {
        Ok(report)
    }
}

fn collect_storage_disks(
    debug_enabled: bool,
    debug: &mut Vec<DebugRecord>,
) -> Result<Vec<MsftPhysicalDisk>, String> {
    let storage = connect_namespace("ROOT\\Microsoft\\Windows\\Storage")?;
    let full_query = "SELECT DeviceId, FriendlyName, Model, SerialNumber, Size, MediaType, BusType, FirmwareVersion, HealthStatus FROM MSFT_PhysicalDisk";

    match raw_query::<MsftPhysicalDisk>(&storage, full_query) {
        Ok(rows) => {
            if debug_enabled {
                debug.push(
                    DebugRecord::new("MSFT_PhysicalDisk", "native-wmi")
                        .field("query", full_query)
                        .field("rows", rows.len().to_string()),
                );
            }
            Ok(rows)
        }
        Err(full_err) => {
            let base_query = "SELECT DeviceId, FriendlyName, SerialNumber, Size, MediaType, BusType, FirmwareVersion, HealthStatus FROM MSFT_PhysicalDisk";
            let rows = raw_query::<MsftPhysicalDisk>(&storage, base_query).map_err(|base_err| {
                format!(
                    "full MSFT_PhysicalDisk query failed: {full_err}; base query failed: {base_err}"
                )
            })?;

            if debug_enabled {
                debug.push(
                    DebugRecord::new("MSFT_PhysicalDisk", "native-wmi")
                        .field("query", base_query)
                        .field("rows", rows.len().to_string())
                        .note(format!(
                            "Model query failed; used base query: {full_err}"
                        )),
                );
            }
            Ok(rows)
        }
    }
}

fn collect_disk_drive_rows(
    conn: &WMIConnection,
    warnings: &mut Vec<HdrtWarning>,
) -> Vec<Win32DiskDrive> {
    match raw_query::<Win32DiskDrive>(
        conn,
        "SELECT DeviceID, Index, Model, SerialNumber, Size, MediaType, InterfaceType, FirmwareRevision FROM Win32_DiskDrive",
    ) {
        Ok(rows) => rows,
        Err(err) => {
            warnings.push(HdrtWarning::with_hint(
                "windows-native-wmi-disk-unavailable",
                err,
                "Falling back may leave disk inventory incomplete.",
            ));
            Vec::new()
        }
    }
}

fn collect_storage_descriptors(
    storage_disks: &[MsftPhysicalDisk],
    disk_drives: &[Win32DiskDrive],
    debug_enabled: bool,
    debug: &mut Vec<DebugRecord>,
) -> Vec<StorageDescriptor> {
    let mut indexes = BTreeSet::new();

    for disk in storage_disks {
        if let Some(index) = disk.device_id.as_deref().and_then(parse_physical_index) {
            indexes.insert(index);
        }
    }
    for disk in disk_drives {
        if let Some(index) = disk_drive_index(disk) {
            indexes.insert(index);
        }
    }

    indexes
        .into_iter()
        .filter_map(|index| match super::native_storage::query_physical_drive(index) {
            Ok(descriptor) => {
                if debug_enabled {
                    debug.push(
                        DebugRecord::new(
                            descriptor.device.clone(),
                            "DeviceIoControl(IOCTL_STORAGE_QUERY_PROPERTY)",
                        )
                        .field("vendor", descriptor.vendor.clone())
                        .field("product", descriptor.product.clone())
                        .field("revision", descriptor.revision.clone())
                        .field("serial", descriptor.serial.clone())
                        .field("bus", descriptor.bus.clone())
                        .field("raw_size", descriptor.raw_size.to_string()),
                    );
                }
                Some(descriptor)
            }
            Err(err) => {
                if debug_enabled {
                    debug.push(
                        DebugRecord::new(
                            format!("PhysicalDrive{index}"),
                            "DeviceIoControl(IOCTL_STORAGE_QUERY_PROPERTY)",
                        )
                        .field("status", "failed")
                        .note(err),
                    );
                }
                None
            }
        })
        .collect()
}

fn storage_rows_to_disks(
    storage_disks: &[MsftPhysicalDisk],
    disk_drives: &[Win32DiskDrive],
    descriptors: &[StorageDescriptor],
    debug_enabled: bool,
    debug: &mut Vec<DebugRecord>,
) -> Vec<DiskInfo> {
    storage_disks
        .iter()
        .map(|disk| {
            let index = disk.device_id.as_deref().and_then(parse_physical_index);
            let win32 = find_disk_drive_for_storage(disk_drives, disk, index);
            let descriptor_index = index.or_else(|| win32.and_then(disk_drive_index));
            let descriptor = descriptor_index.and_then(|index| find_descriptor(descriptors, index));
            let disk = disk_from_storage_row(disk, win32, descriptor);

            if debug_enabled {
                debug.push(debug_final_disk(
                    &disk,
                    descriptor_index,
                    win32.is_some(),
                    descriptor.is_some(),
                ));
            }

            disk
        })
        .collect()
}

fn disk_drive_rows_to_disks(
    disk_drives: &[Win32DiskDrive],
    descriptors: &[StorageDescriptor],
    debug_enabled: bool,
    debug: &mut Vec<DebugRecord>,
) -> Vec<DiskInfo> {
    disk_drives
        .iter()
        .map(|disk| {
            let index = disk_drive_index(disk);
            let descriptor = index.and_then(|index| find_descriptor(descriptors, index));
            let disk = disk_from_disk_drive_row(disk, descriptor);

            if debug_enabled {
                debug.push(debug_final_disk(
                    &disk,
                    index,
                    true,
                    descriptor.is_some(),
                ));
            }

            disk
        })
        .collect()
}

fn disk_from_storage_row(
    disk: &MsftPhysicalDisk,
    win32: Option<&Win32DiskDrive>,
    descriptor: Option<&StorageDescriptor>,
) -> DiskInfo {
    let model = first_known(&[
        known(disk.friendly_name.clone()),
        known(disk.model.clone()),
        win32
            .map(|disk| known(disk.model.clone()))
            .unwrap_or_else(unknown),
        descriptor
            .map(|descriptor| descriptor.product.clone())
            .unwrap_or_else(unknown),
    ]);
    let source = joined_source(&[
        Some("native-wmi/MSFT_PhysicalDisk"),
        win32.map(|_| "native-wmi/Win32_DiskDrive"),
        descriptor.map(|_| "DeviceIoControl/storage-descriptor"),
    ]);

    DiskInfo {
        device: known(disk.device_id.clone()),
        model,
        serial: first_known(&[
            known(disk.serial_number.clone()),
            win32
                .map(|disk| known(disk.serial_number.clone()))
                .unwrap_or_else(unknown),
            descriptor
                .map(|descriptor| descriptor.serial.clone())
                .unwrap_or_else(unknown),
        ]),
        size: disk
            .size
            .or_else(|| win32.and_then(|disk| disk.size))
            .map(format_bytes)
            .unwrap_or_else(unknown),
        media_type: first_known(&[
            physical_media_type(disk.media_type),
            win32
                .map(|disk| known(disk.media_type.clone()))
                .unwrap_or_else(unknown),
        ]),
        bus: first_known(&[
            storage_bus_type(disk.bus_type),
            descriptor
                .map(|descriptor| descriptor.bus.clone())
                .unwrap_or_else(unknown),
            win32
                .map(|disk| known(disk.interface_type.clone()))
                .unwrap_or_else(unknown),
        ]),
        firmware: first_known(&[
            known(disk.firmware_version.clone()),
            win32
                .map(|disk| known(disk.firmware_revision.clone()))
                .unwrap_or_else(unknown),
            descriptor
                .map(|descriptor| descriptor.revision.clone())
                .unwrap_or_else(unknown),
        ]),
        health: health_status(disk.health_status),
        source,
        ..DiskInfo::default()
    }
}

fn disk_from_disk_drive_row(
    disk: &Win32DiskDrive,
    descriptor: Option<&StorageDescriptor>,
) -> DiskInfo {
    let index = disk_drive_index(disk);
    let model = first_known(&[
        known(disk.model.clone()),
        descriptor
            .map(|descriptor| descriptor.product.clone())
            .unwrap_or_else(unknown),
    ]);
    DiskInfo {
        device: first_known(&[
            known(disk.device_id.clone()),
            index
                .map(|index| format!("PhysicalDrive{index}"))
                .unwrap_or_else(unknown),
        ]),
        model,
        serial: first_known(&[
            known(disk.serial_number.clone()),
            descriptor
                .map(|descriptor| descriptor.serial.clone())
                .unwrap_or_else(unknown),
        ]),
        size: disk.size.map(format_bytes).unwrap_or_else(unknown),
        media_type: known(disk.media_type.clone()),
        bus: first_known(&[
            descriptor
                .map(|descriptor| descriptor.bus.clone())
                .unwrap_or_else(unknown),
            known(disk.interface_type.clone()),
        ]),
        firmware: first_known(&[
            known(disk.firmware_revision.clone()),
            descriptor
                .map(|descriptor| descriptor.revision.clone())
                .unwrap_or_else(unknown),
        ]),
        source: joined_source(&[
            Some("native-wmi/Win32_DiskDrive"),
            descriptor.map(|_| "DeviceIoControl/storage-descriptor"),
        ]),
        ..DiskInfo::default()
    }
}

fn debug_final_disk(
    disk: &DiskInfo,
    index: Option<u32>,
    has_win32: bool,
    has_descriptor: bool,
) -> DebugRecord {
    DebugRecord::new(disk.device.clone(), "final-disk-merge")
        .field(
            "physical_index",
            index
                .map(|index| index.to_string())
                .unwrap_or_else(unknown),
        )
        .field("model", disk.model.clone())
        .field("serial", disk.serial.clone())
        .field("size", disk.size.clone())
        .field("media_type", disk.media_type.clone())
        .field("bus", disk.bus.clone())
        .field("firmware", disk.firmware.clone())
        .field("health", disk.health.clone())
        .field("source", disk.source.clone())
        .field("win32_joined", has_win32.to_string())
        .field("descriptor_joined", has_descriptor.to_string())
}

fn find_disk_drive(disks: &[Win32DiskDrive], index: u32) -> Option<&Win32DiskDrive> {
    disks
        .iter()
        .find(|disk| disk_drive_index(disk).is_some_and(|candidate| candidate == index))
}

fn find_disk_drive_for_storage<'a>(
    disks: &'a [Win32DiskDrive],
    storage: &MsftPhysicalDisk,
    index: Option<u32>,
) -> Option<&'a Win32DiskDrive> {
    if let Some(disk) = index.and_then(|index| find_disk_drive(disks, index)) {
        return Some(disk);
    }

    let serial = known(storage.serial_number.clone());
    if serial != "Unknown" {
        if let Some(disk) = disks.iter().find(|disk| {
            known(disk.serial_number.clone()).eq_ignore_ascii_case(serial.trim())
        }) {
            return Some(disk);
        }
    }

    let model = first_known(&[
        known(storage.friendly_name.clone()),
        known(storage.model.clone()),
    ]);
    if model != "Unknown" {
        return disks.iter().find(|disk| {
            known(disk.model.clone()).eq_ignore_ascii_case(model.trim())
        });
    }

    None
}

fn find_descriptor(descriptors: &[StorageDescriptor], index: u32) -> Option<&StorageDescriptor> {
    descriptors
        .iter()
        .find(|descriptor| descriptor.index == index)
}

fn disk_drive_index(disk: &Win32DiskDrive) -> Option<u32> {
    disk.index
        .or_else(|| disk.device_id.as_deref().and_then(parse_physical_index))
}

fn parse_physical_index(value: &str) -> Option<u32> {
    let value = value.trim();
    if let Ok(index) = value.parse::<u32>() {
        return Some(index);
    }

    let upper = value.to_ascii_uppercase();
    let marker = "PHYSICALDRIVE";
    upper.find(marker).and_then(|start| {
        let suffix = &value[start + marker.len()..];
        let digits = suffix
            .chars()
            .take_while(|ch| ch.is_ascii_digit())
            .collect::<String>();
        digits.parse().ok()
    })
}

fn joined_source(parts: &[Option<&str>]) -> String {
    let joined = parts
        .iter()
        .flatten()
        .copied()
        .collect::<Vec<_>>()
        .join(" + ");
    if joined.is_empty() {
        unknown()
    } else {
        joined
    }
}

fn connect_namespace(namespace: &str) -> Result<WMIConnection, String> {
    WMIConnection::with_namespace_path(namespace)
        .map_err(|err| format!("failed to connect to {namespace}: {err}"))
}

fn raw_query<T>(conn: &WMIConnection, query: &str) -> Result<Vec<T>, String>
where
    T: DeserializeOwned,
{
    conn.raw_query(query)
        .map_err(|err| format!("WMI query failed `{query}`: {err}"))
}

fn known(value: Option<String>) -> String {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "Unknown".to_string())
}

fn physical_media_type(value: Option<u16>) -> String {
    match value {
        Some(3) => "HDD".to_string(),
        Some(4) => "SSD".to_string(),
        Some(5) => "SCM".to_string(),
        Some(0) | None => "Unspecified".to_string(),
        Some(value) => format!("MediaType({value})"),
    }
}

fn storage_bus_type(value: Option<u16>) -> String {
    match value {
        Some(1) => "SCSI".to_string(),
        Some(2) => "ATAPI".to_string(),
        Some(3) => "ATA".to_string(),
        Some(4) => "IEEE 1394".to_string(),
        Some(5) => "SSA".to_string(),
        Some(6) => "Fibre Channel".to_string(),
        Some(7) => "USB".to_string(),
        Some(8) => "RAID".to_string(),
        Some(9) => "iSCSI".to_string(),
        Some(10) => "SAS".to_string(),
        Some(11) => "SATA".to_string(),
        Some(12) => "SD".to_string(),
        Some(13) => "MMC".to_string(),
        Some(15) => "FileBackedVirtual".to_string(),
        Some(16) => "Storage Spaces".to_string(),
        Some(17) => "NVMe".to_string(),
        Some(18) => "SCM".to_string(),
        Some(19) => "UFS".to_string(),
        Some(0) | None => "Unknown".to_string(),
        Some(value) => format!("BusType({value})"),
    }
}

fn health_status(value: Option<u16>) -> String {
    match value {
        Some(0) => "Healthy".to_string(),
        Some(1) => "Warning".to_string(),
        Some(2) => "Unhealthy".to_string(),
        Some(5) | None => "Unknown".to_string(),
        Some(value) => format!("HealthStatus({value})"),
    }
}
