use std::fs;

use crate::collector::capability;
use crate::hardware::{unknown, HdrtWarning, MotherboardInfo};

use super::command::non_empty_or_unknown;

pub(super) fn collect() -> Option<MotherboardInfo> {
    let read_dmi = |name: &str| -> String {
        fs::read_to_string(format!("/sys/class/dmi/id/{name}"))
            .map(|value| non_empty_or_unknown(value.trim()))
            .unwrap_or_else(|_| unknown())
    };

    Some(MotherboardInfo {
        manufacturer: read_dmi("board_vendor"),
        product: read_dmi("board_name"),
        version: read_dmi("board_version"),
        serial: read_dmi("board_serial"),
        bios_vendor: read_dmi("bios_vendor"),
        bios_version: read_dmi("bios_version"),
        source: "/sys/class/dmi/id".to_string(),
        warnings: if capability::is_elevated() {
            Vec::new()
        } else {
            vec![HdrtWarning::with_hint(
                "dmi-permission",
                "Some DMI fields may be hidden without root privileges.",
                "Run sudo hdrt motherboard for more complete board details.",
            )]
        },
    })
}
