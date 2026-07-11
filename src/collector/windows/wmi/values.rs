pub(super) fn physical_media_type(value: Option<u16>) -> String {
    match value {
        Some(3) => "HDD".to_string(),
        Some(4) => "SSD".to_string(),
        Some(5) => "SCM".to_string(),
        Some(0) | None => "Unspecified".to_string(),
        Some(value) => format!("MediaType({value})"),
    }
}

pub(super) fn storage_bus_type(value: Option<u16>) -> String {
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

pub(super) fn health_status(value: Option<u16>) -> String {
    match value {
        Some(0) => "Healthy".to_string(),
        Some(1) => "Warning".to_string(),
        Some(2) => "Unhealthy".to_string(),
        Some(5) | None => "Unknown".to_string(),
        Some(value) => format!("HealthStatus({value})"),
    }
}
