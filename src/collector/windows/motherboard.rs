use serde_json::Value;

use crate::hardware::MotherboardInfo;

use super::util::{first_known, value_string};

pub fn collect(root: &Value) -> Option<MotherboardInfo> {
    let base_board = root.get("BaseBoard")?;
    let bios = root.get("Bios").unwrap_or(&Value::Null);

    Some(MotherboardInfo {
        manufacturer: value_string(base_board, "Manufacturer"),
        product: value_string(base_board, "Product"),
        version: value_string(base_board, "Version"),
        serial: value_string(base_board, "SerialNumber"),
        bios_vendor: value_string(bios, "Manufacturer"),
        bios_version: first_known(&[
            value_string(bios, "SMBIOSBIOSVersion"),
            value_string(bios, "Version"),
        ]),
        source: "Win32_BaseBoard + Win32_BIOS".to_string(),
        ..MotherboardInfo::default()
    })
}
