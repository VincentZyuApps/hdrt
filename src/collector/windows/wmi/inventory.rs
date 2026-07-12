use wmi::WMIConnection;

use crate::hardware::{CpuInfo, HdrtWarning, MemoryDevice, MotherboardInfo};

use super::super::util::{first_known, format_bytes};
use super::rows::{Win32BaseBoard, Win32Bios, Win32PhysicalMemory, Win32Processor};
use super::{known, raw_query};

pub(super) fn collect_memory(
    conn: &WMIConnection,
    warnings: &mut Vec<HdrtWarning>,
) -> Vec<MemoryDevice> {
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

pub(super) fn collect_cpu(
    conn: &WMIConnection,
    warnings: &mut Vec<HdrtWarning>,
) -> Option<CpuInfo> {
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

pub(super) fn collect_motherboard(
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
            .map(|bios| first_known(&[known(bios.smbios_bios_version), known(bios.version)]))
            .unwrap_or_else(|| "Unknown".to_string()),
        source: "native-wmi/Win32_BaseBoard+Win32_BIOS".to_string(),
        ..MotherboardInfo::default()
    })
}
