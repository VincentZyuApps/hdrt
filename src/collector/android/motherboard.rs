use std::collections::HashMap;

use crate::hardware::{unknown, HdrtWarning, MotherboardInfo};

use super::command::{non_empty_or_unknown, parse_getprop, run_command};

pub(super) fn collect() -> Option<MotherboardInfo> {
    let props = match run_command("getprop", &[]) {
        Ok(output) => parse_getprop(&output),
        Err(err) => {
            return Some(MotherboardInfo {
                source: "getprop".to_string(),
                warnings: vec![HdrtWarning::with_hint(
                    "android-getprop-unavailable",
                    format!("Could not run getprop to collect Android device properties: {err}"),
                    "Run hdrt inside Android or Termux for Android device properties.",
                )],
                ..MotherboardInfo::default()
            });
        }
    };

    Some(MotherboardInfo {
        manufacturer: prop(&props, &["ro.product.manufacturer", "ro.product.vendor.manufacturer"]),
        product: prop(&props, &["ro.product.model", "ro.product.vendor.model"]),
        version: prop(&props, &["ro.product.board", "ro.boot.hardware", "ro.hardware"]),
        serial: prop(&props, &["ro.serialno", "ro.boot.serialno"]),
        bios_vendor: "Android".to_string(),
        bios_version: prop(&props, &["ro.build.version.release", "ro.build.version.incremental"]),
        source: "getprop".to_string(),
        warnings: vec![HdrtWarning::with_hint(
            "android-limited-hardware-fields",
            "Android exposes device properties rather than desktop SMBIOS/DMI fields.",
            "Some board, serial, and firmware fields may be hidden by Android privacy restrictions.",
        )],
    })
}

fn prop(props: &HashMap<String, String>, keys: &[&str]) -> String {
    keys.iter()
        .find_map(|key| props.get(*key))
        .map(|value| non_empty_or_unknown(value))
        .unwrap_or_else(unknown)
}
