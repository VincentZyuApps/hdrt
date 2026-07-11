use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(super) struct MsftPhysicalDisk {
    #[serde(rename = "DeviceId")]
    pub(super) device_id: Option<String>,
    #[serde(rename = "FriendlyName")]
    pub(super) friendly_name: Option<String>,
    #[serde(rename = "Model")]
    pub(super) model: Option<String>,
    #[serde(rename = "SerialNumber")]
    pub(super) serial_number: Option<String>,
    #[serde(rename = "Size")]
    pub(super) size: Option<u64>,
    #[serde(rename = "MediaType")]
    pub(super) media_type: Option<u16>,
    #[serde(rename = "BusType")]
    pub(super) bus_type: Option<u16>,
    #[serde(rename = "FirmwareVersion")]
    pub(super) firmware_version: Option<String>,
    #[serde(rename = "HealthStatus")]
    pub(super) health_status: Option<u16>,
}

#[derive(Debug, Deserialize)]
pub(super) struct Win32DiskDrive {
    #[serde(rename = "DeviceID")]
    pub(super) device_id: Option<String>,
    #[serde(rename = "Index")]
    pub(super) index: Option<u32>,
    #[serde(rename = "Model")]
    pub(super) model: Option<String>,
    #[serde(rename = "SerialNumber")]
    pub(super) serial_number: Option<String>,
    #[serde(rename = "Size")]
    pub(super) size: Option<u64>,
    #[serde(rename = "MediaType")]
    pub(super) media_type: Option<String>,
    #[serde(rename = "InterfaceType")]
    pub(super) interface_type: Option<String>,
    #[serde(rename = "FirmwareRevision")]
    pub(super) firmware_revision: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(super) struct Win32PhysicalMemory {
    #[serde(rename = "BankLabel")]
    pub(super) bank_label: Option<String>,
    #[serde(rename = "DeviceLocator")]
    pub(super) device_locator: Option<String>,
    #[serde(rename = "Capacity")]
    pub(super) capacity: Option<u64>,
    #[serde(rename = "Speed")]
    pub(super) speed: Option<u32>,
    #[serde(rename = "ConfiguredClockSpeed")]
    pub(super) configured_clock_speed: Option<u32>,
    #[serde(rename = "Manufacturer")]
    pub(super) manufacturer: Option<String>,
    #[serde(rename = "PartNumber")]
    pub(super) part_number: Option<String>,
    #[serde(rename = "SerialNumber")]
    pub(super) serial_number: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(super) struct Win32Processor {
    #[serde(rename = "Name")]
    pub(super) name: Option<String>,
    #[serde(rename = "Manufacturer")]
    pub(super) manufacturer: Option<String>,
    #[serde(rename = "NumberOfCores")]
    pub(super) number_of_cores: Option<u32>,
    #[serde(rename = "NumberOfLogicalProcessors")]
    pub(super) number_of_logical_processors: Option<u32>,
    #[serde(rename = "MaxClockSpeed")]
    pub(super) max_clock_speed: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub(super) struct Win32BaseBoard {
    #[serde(rename = "Manufacturer")]
    pub(super) manufacturer: Option<String>,
    #[serde(rename = "Product")]
    pub(super) product: Option<String>,
    #[serde(rename = "Version")]
    pub(super) version: Option<String>,
    #[serde(rename = "SerialNumber")]
    pub(super) serial_number: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(super) struct Win32Bios {
    #[serde(rename = "Manufacturer")]
    pub(super) manufacturer: Option<String>,
    #[serde(rename = "SMBIOSBIOSVersion")]
    pub(super) smbios_bios_version: Option<String>,
    #[serde(rename = "Version")]
    pub(super) version: Option<String>,
}
