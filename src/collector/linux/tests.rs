use crate::hardware::{unknown, DiskInfo};

use super::merge_disk;

fn complete_disk(device: &str, model: &str, source: &str) -> DiskInfo {
    DiskInfo {
        device: device.to_string(),
        model: model.to_string(),
        serial: "SERIAL".to_string(),
        size: "100.00 GiB".to_string(),
        media_type: "SSD".to_string(),
        bus: "SATA".to_string(),
        firmware: "1.0".to_string(),
        health: "Healthy".to_string(),
        source: source.to_string(),
        warnings: Vec::new(),
    }
}

#[test]
fn shell_merge_fills_unknown_native_fields() {
    let mut native = complete_disk("sda", "Native Model", "native");
    native.serial = unknown();
    let shell = complete_disk("sda", "Shell Model", "shell");

    merge_disk(&mut native, shell);

    assert_eq!(native.model, "Native Model");
    assert_eq!(native.serial, "SERIAL");
    assert_eq!(native.source, "native + shell");
}
