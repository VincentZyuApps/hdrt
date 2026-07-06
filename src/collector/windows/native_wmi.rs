use serde::de::DeserializeOwned;
use serde::Deserialize;
use wmi::WMIConnection;

use crate::hardware::{CpuInfo, DiskInfo, HardwareReport, HdrtWarning, MemoryDevice, MotherboardInfo};

use super::util::{clean_manufacturer, first_known, format_bytes};

pub fn collect_report() -> Result<HardwareReport, String> {
    let cimv2 = connect_namespace("ROOT\\CIMV2")?;
    let mut warnings = Vec::new();

    let disks = match collect_storage_disks() {
        Ok(disks) if !disks.is_empty() => disks,
        Ok(_) => collect_disk_drives(&cimv2, &mut warnings),
        Err(err) => {
            warnings.push(HdrtWarning::with_hint(
                "windows-native-wmi-storage-unavailable",
                err,
                "Falling back to Win32_DiskDrive for disk inventory.",
            ));
            collect_disk_drives(&cimv2, &mut warnings)
        }
    };

    let memory = collect_memory(&cimv2, &mut warnings);
    let cpu = collect_cpu(&cimv2, &mut warnings);
    let motherboard = collect_motherboard(&cimv2, &mut warnings);

    let report = HardwareReport {
        disks,
        memory,
        cpu,
        motherboard,
        warnings,
    };

    if report.disks.is_empty()
        && report.memory.is_empty()
        && report.cpu.is_none()
        && report.motherboard.is_none()
    {
        Err("native WMI returned no hardware data".to_string())
    } else {
        Ok(report)
    }
}

fn collect_storage_disks() -> Result<Vec<DiskInfo>, String> {
    let storage = connect_namespace("ROOT\\Microsoft\\Windows\\Storage")?;
    let rows: Vec<MsftPhysicalDisk> = raw_query(
        &storage,
        "SELECT DeviceId, FriendlyName, SerialNumber, Size, MediaType, BusType, FirmwareVersion, HealthStatus FROM MSFT_PhysicalDisk",
    )?;

    Ok(rows
        .into_iter()
        .map(|disk| DiskInfo {
            device: known(disk.device_id),
            model: known(disk.friendly_name),
            serial: known(disk.serial_number),
            size: disk
                .size
                .map(format_bytes)
                .unwrap_or_else(|| "Unknown".to_string()),
            media_type: physical_media_type(disk.media_type),
            bus: storage_bus_type(disk.bus_type),
            firmware: known(disk.firmware_version),
            health: health_status(disk.health_status),
            source: "native-wmi/MSFT_PhysicalDisk".to_string(),
            ..DiskInfo::default()
        })
        .collect())
}

fn collect_disk_drives(conn: &WMIConnection, warnings: &mut Vec<HdrtWarning>) -> Vec<DiskInfo> {
    match raw_query::<Win32DiskDrive>(
        conn,
        "SELECT DeviceID, Index, Model, SerialNumber, Size, MediaType, InterfaceType, FirmwareRevision, Manufacturer FROM Win32_DiskDrive",
    ) {
        Ok(rows) => rows
            .into_iter()
            .map(|disk| {
                let device = first_known(&[
                    known(disk.device_id),
                    disk.index
                        .map(|index| format!("PhysicalDrive{index}"))
                        .unwrap_or_else(|| "Unknown".to_string()),
                ]);
                DiskInfo {
                    device,
                    model: known(disk.model),
                    brand: clean_manufacturer(&known(disk.manufacturer)),
                    serial: known(disk.serial_number),
                    size: disk
                        .size
                        .map(format_bytes)
                        .unwrap_or_else(|| "Unknown".to_string()),
                    media_type: known(disk.media_type),
                    bus: known(disk.interface_type),
                    firmware: known(disk.firmware_revision),
                    source: "native-wmi/Win32_DiskDrive".to_string(),
                    ..DiskInfo::default()
                }
            })
            .collect(),
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

fn collect_memory(conn: &WMIConnection, warnings: &mut Vec<HdrtWarning>) -> Vec<MemoryDevice> {
    match raw_query::<Win32PhysicalMemory>(
        conn,
        "SELECT BankLabel, DeviceLocator, Capacity, Speed, ConfiguredClockSpeed, Manufacturer, PartNumber, SerialNumber FROM Win32_PhysicalMemory",
    ) {
        Ok(rows) => rows
            .into_iter()
            .map(|memory| {
                let speed = memory.configured_clock_speed.or(memory.speed);
                MemoryDevice {
                    slot: first_known(&[known(memory.device_locator), known(memory.bank_label)]),
                    size: memory
                        .capacity
                        .map(format_bytes)
                        .unwrap_or_else(|| "Unknown".to_string()),
                    speed: speed
                        .map(|value| format!("{value} MT/s"))
                        .unwrap_or_else(|| "Unknown".to_string()),
                    manufacturer: known(memory.manufacturer),
                    part_number: known(memory.part_number),
                    serial: known(memory.serial_number),
                    source: "native-wmi/Win32_PhysicalMemory".to_string(),
                    ..MemoryDevice::default()
                }
            })
            .collect(),
        Err(err) => {
            warnings.push(HdrtWarning::with_hint(
                "windows-native-wmi-memory-unavailable",
                err,
                "Memory module inventory is unavailable from native WMI.",
            ));
            Vec::new()
        }
    }
}

fn collect_cpu(conn: &WMIConnection, warnings: &mut Vec<HdrtWarning>) -> Option<CpuInfo> {
    match raw_query::<Win32Processor>(
        conn,
        "SELECT Name, Manufacturer, NumberOfCores, NumberOfLogicalProcessors, MaxClockSpeed FROM Win32_Processor",
    ) {
        Ok(rows) => rows.into_iter().next().map(|cpu| CpuInfo {
            model: known(cpu.name),
            vendor: known(cpu.manufacturer),
            physical_cores: cpu.number_of_cores.map(|value| value as usize),
            logical_threads: cpu
                .number_of_logical_processors
                .map(|value| value as usize),
            frequency: cpu
                .max_clock_speed
                .map(|value| format!("{value} MHz"))
                .unwrap_or_else(|| "Unknown".to_string()),
            source: "native-wmi/Win32_Processor".to_string(),
            ..CpuInfo::default()
        }),
        Err(err) => {
            warnings.push(HdrtWarning::with_hint(
                "windows-native-wmi-cpu-unavailable",
                err,
                "CPU inventory is unavailable from native WMI.",
            ));
            None
        }
    }
}

fn collect_motherboard(
    conn: &WMIConnection,
    warnings: &mut Vec<HdrtWarning>,
) -> Option<MotherboardInfo> {
    let board = match raw_query::<Win32BaseBoard>(
        conn,
        "SELECT Manufacturer, Product, Version, SerialNumber FROM Win32_BaseBoard",
    ) {
        Ok(rows) => rows.into_iter().next(),
        Err(err) => {
            warnings.push(HdrtWarning::with_hint(
                "windows-native-wmi-baseboard-unavailable",
                err,
                "Baseboard inventory is unavailable from native WMI.",
            ));
            None
        }
    };

    let bios = match raw_query::<Win32Bios>(
        conn,
        "SELECT Manufacturer, SMBIOSBIOSVersion, Version FROM Win32_BIOS",
    ) {
        Ok(rows) => rows.into_iter().next(),
        Err(err) => {
            warnings.push(HdrtWarning::with_hint(
                "windows-native-wmi-bios-unavailable",
                err,
                "BIOS inventory is unavailable from native WMI.",
            ));
            None
        }
    };

    if board.is_none() && bios.is_none() {
        return None;
    }

    Some(MotherboardInfo {
        manufacturer: board
            .as_ref()
            .map(|board| known(board.manufacturer.clone()))
            .unwrap_or_else(|| "Unknown".to_string()),
        product: board
            .as_ref()
            .map(|board| known(board.product.clone()))
            .unwrap_or_else(|| "Unknown".to_string()),
        version: board
            .as_ref()
            .map(|board| known(board.version.clone()))
            .unwrap_or_else(|| "Unknown".to_string()),
        serial: board
            .as_ref()
            .map(|board| known(board.serial_number.clone()))
            .unwrap_or_else(|| "Unknown".to_string()),
        bios_vendor: bios
            .as_ref()
            .map(|bios| known(bios.manufacturer.clone()))
            .unwrap_or_else(|| "Unknown".to_string()),
        bios_version: bios
            .map(|bios| {
                first_known(&[known(bios.smbios_bios_version), known(bios.version)])
            })
            .unwrap_or_else(|| "Unknown".to_string()),
        source: "native-wmi/Win32_BaseBoard+Win32_BIOS".to_string(),
        ..MotherboardInfo::default()
    })
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

#[derive(Debug, Deserialize)]
struct MsftPhysicalDisk {
    #[serde(rename = "DeviceId")]
    device_id: Option<String>,
    #[serde(rename = "FriendlyName")]
    friendly_name: Option<String>,
    #[serde(rename = "SerialNumber")]
    serial_number: Option<String>,
    #[serde(rename = "Size")]
    size: Option<u64>,
    #[serde(rename = "MediaType")]
    media_type: Option<u16>,
    #[serde(rename = "BusType")]
    bus_type: Option<u16>,
    #[serde(rename = "FirmwareVersion")]
    firmware_version: Option<String>,
    #[serde(rename = "HealthStatus")]
    health_status: Option<u16>,
}

#[derive(Debug, Deserialize)]
struct Win32DiskDrive {
    #[serde(rename = "DeviceID")]
    device_id: Option<String>,
    #[serde(rename = "Index")]
    index: Option<u32>,
    #[serde(rename = "Model")]
    model: Option<String>,
    #[serde(rename = "SerialNumber")]
    serial_number: Option<String>,
    #[serde(rename = "Size")]
    size: Option<u64>,
    #[serde(rename = "MediaType")]
    media_type: Option<String>,
    #[serde(rename = "InterfaceType")]
    interface_type: Option<String>,
    #[serde(rename = "FirmwareRevision")]
    firmware_revision: Option<String>,
    #[serde(rename = "Manufacturer")]
    manufacturer: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Win32PhysicalMemory {
    #[serde(rename = "BankLabel")]
    bank_label: Option<String>,
    #[serde(rename = "DeviceLocator")]
    device_locator: Option<String>,
    #[serde(rename = "Capacity")]
    capacity: Option<u64>,
    #[serde(rename = "Speed")]
    speed: Option<u32>,
    #[serde(rename = "ConfiguredClockSpeed")]
    configured_clock_speed: Option<u32>,
    #[serde(rename = "Manufacturer")]
    manufacturer: Option<String>,
    #[serde(rename = "PartNumber")]
    part_number: Option<String>,
    #[serde(rename = "SerialNumber")]
    serial_number: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Win32Processor {
    #[serde(rename = "Name")]
    name: Option<String>,
    #[serde(rename = "Manufacturer")]
    manufacturer: Option<String>,
    #[serde(rename = "NumberOfCores")]
    number_of_cores: Option<u32>,
    #[serde(rename = "NumberOfLogicalProcessors")]
    number_of_logical_processors: Option<u32>,
    #[serde(rename = "MaxClockSpeed")]
    max_clock_speed: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct Win32BaseBoard {
    #[serde(rename = "Manufacturer")]
    manufacturer: Option<String>,
    #[serde(rename = "Product")]
    product: Option<String>,
    #[serde(rename = "Version")]
    version: Option<String>,
    #[serde(rename = "SerialNumber")]
    serial_number: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Win32Bios {
    #[serde(rename = "Manufacturer")]
    manufacturer: Option<String>,
    #[serde(rename = "SMBIOSBIOSVersion")]
    smbios_bios_version: Option<String>,
    #[serde(rename = "Version")]
    version: Option<String>,
}
