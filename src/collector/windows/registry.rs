use winreg::enums::{HKEY_LOCAL_MACHINE, KEY_READ};
use winreg::RegKey;

use crate::hardware::{CpuInfo, DiskInfo, MotherboardInfo};

use super::util::first_known;

const CPU_KEY: &str = r"HARDWARE\DESCRIPTION\System\CentralProcessor\0";
const CPU_ROOT_KEY: &str = r"HARDWARE\DESCRIPTION\System\CentralProcessor";
const BIOS_KEY: &str = r"HARDWARE\DESCRIPTION\System\BIOS";
const DISK_ENUM_KEY: &str = r"SYSTEM\CurrentControlSet\Services\disk\Enum";

pub fn physical_disks() -> Vec<DiskInfo> {
    let count = read_u32(DISK_ENUM_KEY, "Count").unwrap_or(0);
    (0..count)
        .filter_map(|index| read_string(DISK_ENUM_KEY, &index.to_string()).map(|id| (index, id)))
        .map(|(index, id)| disk_from_pnp_id(index, &id))
        .collect()
}

pub fn cpu_info() -> Option<CpuInfo> {
    let model = read_string(CPU_KEY, "ProcessorNameString");
    let vendor = read_string(CPU_KEY, "VendorIdentifier");
    let frequency = read_u32(CPU_KEY, "~MHz").map(|mhz| format!("{mhz} MHz"));

    let cpu = CpuInfo {
        model: model.unwrap_or_else(|| "Unknown".to_string()),
        vendor: vendor.unwrap_or_else(|| "Unknown".to_string()),
        logical_threads: logical_thread_count(),
        frequency: frequency.unwrap_or_else(|| "Unknown".to_string()),
        source: "registry".to_string(),
        ..CpuInfo::default()
    };

    if cpu.model == "Unknown" && cpu.vendor == "Unknown" && cpu.frequency == "Unknown" {
        None
    } else {
        Some(cpu)
    }
}

pub fn motherboard_info() -> Option<MotherboardInfo> {
    let board = MotherboardInfo {
        manufacturer: first_known(&[
            read_string(BIOS_KEY, "BaseBoardManufacturer").unwrap_or_else(|| "Unknown".to_string()),
            read_string(BIOS_KEY, "SystemManufacturer").unwrap_or_else(|| "Unknown".to_string()),
        ]),
        product: first_known(&[
            read_string(BIOS_KEY, "BaseBoardProduct").unwrap_or_else(|| "Unknown".to_string()),
            read_string(BIOS_KEY, "SystemProductName").unwrap_or_else(|| "Unknown".to_string()),
        ]),
        version: read_string(BIOS_KEY, "BaseBoardVersion").unwrap_or_else(|| "Unknown".to_string()),
        serial: read_string(BIOS_KEY, "BaseBoardSerialNumber")
            .unwrap_or_else(|| "Unknown".to_string()),
        bios_vendor: read_string(BIOS_KEY, "BIOSVendor").unwrap_or_else(|| "Unknown".to_string()),
        bios_version: first_known(&[
            read_string(BIOS_KEY, "BIOSVersion").unwrap_or_else(|| "Unknown".to_string()),
            read_multi_string(BIOS_KEY, "BIOSVersion").unwrap_or_else(|| "Unknown".to_string()),
            read_string(BIOS_KEY, "SystemBiosVersion").unwrap_or_else(|| "Unknown".to_string()),
        ]),
        source: "registry".to_string(),
        ..MotherboardInfo::default()
    };

    if board.manufacturer == "Unknown"
        && board.product == "Unknown"
        && board.bios_vendor == "Unknown"
        && board.bios_version == "Unknown"
    {
        None
    } else {
        Some(board)
    }
}

fn disk_from_pnp_id(index: u32, id: &str) -> DiskInfo {
    let parts: Vec<&str> = id.split('\\').collect();
    let bus = disk_bus(parts.first().copied().unwrap_or("Unknown"), id);
    let descriptor = parts.get(1).copied().unwrap_or("");
    let serial = parts
        .get(2)
        .map(|value| clean_disk_value(value))
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "Unknown".to_string());

    let brand = disk_component(descriptor, "Ven_");
    let product = disk_component(descriptor, "Prod_");
    let firmware = disk_component(descriptor, "Rev_");
    let model = first_known(&[
        join_known(&brand, &product),
        product.clone(),
        clean_disk_value(descriptor),
        id.to_string(),
    ]);

    DiskInfo {
        device: format!("PhysicalDrive{index}"),
        model,
        brand,
        serial,
        bus,
        firmware,
        media_type: "Physical".to_string(),
        source: "registry".to_string(),
        ..DiskInfo::default()
    }
}

fn disk_component(descriptor: &str, prefix: &str) -> String {
    descriptor
        .split('&')
        .find_map(|part| part.strip_prefix(prefix))
        .map(clean_disk_value)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "Unknown".to_string())
}

fn disk_bus(bus: &str, id: &str) -> String {
    let upper = id.to_ascii_uppercase();
    if upper.starts_with("USBSTOR") {
        "USB".to_string()
    } else if upper.contains("VEN_NVME") || upper.contains("\\NVME") {
        "NVMe".to_string()
    } else if upper.starts_with("SCSI") {
        "SCSI/SATA".to_string()
    } else if upper.starts_with("IDE") {
        "IDE/SATA".to_string()
    } else {
        clean_disk_value(bus)
    }
}

fn clean_disk_value(value: &str) -> String {
    value
        .trim()
        .trim_matches('_')
        .replace('_', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn join_known(left: &str, right: &str) -> String {
    match (left, right) {
        ("Unknown", "Unknown") => "Unknown".to_string(),
        ("Unknown", value) => value.to_string(),
        (value, "Unknown") => value.to_string(),
        (left, right) => format!("{left} {right}"),
    }
}

fn read_string(path: &str, name: &str) -> Option<String> {
    open_hklm(path)
        .and_then(|key| key.get_value::<String, _>(name).ok())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn read_multi_string(path: &str, name: &str) -> Option<String> {
    open_hklm(path)
        .and_then(|key| key.get_value::<Vec<String>, _>(name).ok())
        .map(|values| {
            values
                .into_iter()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
                .collect::<Vec<_>>()
                .join(" ")
        })
        .filter(|value| !value.is_empty())
}

fn read_u32(path: &str, name: &str) -> Option<u32> {
    open_hklm(path).and_then(|key| key.get_value::<u32, _>(name).ok())
}

fn logical_thread_count() -> Option<usize> {
    open_hklm(CPU_ROOT_KEY)
        .map(|key| key.enum_keys().filter_map(Result::ok).count())
        .filter(|count| *count > 0)
}

fn open_hklm(path: &str) -> Option<RegKey> {
    RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey_with_flags(path, KEY_READ)
        .ok()
}
