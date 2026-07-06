use crate::app::options::DetailLevel;
use crate::hardware::{DiskInfo, HdrtWarning};

use super::command::{
    non_empty_or_unknown, parse_key_values, run_command, value_or_unknown,
};

pub(super) fn collect(detail: DetailLevel) -> Vec<DiskInfo> {
    let output = run_command(
        "lsblk",
        &[
            "-d", "-P", "-o", "NAME,MODEL,SERIAL,SIZE,ROTA,TYPE,TRAN,VENDOR,REV",
        ],
    );

    let Ok(output) = output else {
        return vec![DiskInfo {
            warnings: vec![HdrtWarning::with_hint(
                "lsblk-unavailable",
                "Could not run lsblk to collect physical disk inventory.",
                "Install util-linux or run hdrt on a Linux system with lsblk available.",
            )],
            ..DiskInfo::default()
        }];
    };

    let mut disks: Vec<DiskInfo> = output
        .lines()
        .filter_map(|line| {
            let values = parse_key_values(line);
            let name = values.get("NAME")?.to_string();
            let rota = values.get("ROTA").map(String::as_str).unwrap_or("");
            let media_type = match rota {
                "0" => "SSD/NVMe",
                "1" => "HDD",
                _ => "Unknown",
            };

            Some(DiskInfo {
                device: name,
                model: value_or_unknown(values.get("MODEL")),
                serial: value_or_unknown(values.get("SERIAL")),
                size: value_or_unknown(values.get("SIZE")),
                media_type: media_type.to_string(),
                bus: value_or_unknown(values.get("TRAN")),
                firmware: value_or_unknown(values.get("REV")),
                source: "lsblk".to_string(),
                ..DiskInfo::default()
            })
        })
        .collect();

    if matches!(detail, DetailLevel::Smart | DetailLevel::Full) {
        enrich_with_smartctl(&mut disks);
    }

    disks
}

fn enrich_with_smartctl(disks: &mut [DiskInfo]) {
    if which::which("smartctl").is_err() {
        for disk in disks {
            disk.warnings.push(HdrtWarning::with_hint(
                "smartctl-missing",
                "smartctl is not installed, so SMART details are unavailable.",
                "Install smartmontools and run sudo hdrt disk --detail smart.",
            ));
        }
        return;
    }

    for disk in disks {
        let path = format!("/dev/{}", disk.device);
        match run_command("smartctl", &["-a", &path]) {
            Ok(output) => apply_smartctl_output(disk, &output),
            Err(err) => disk.warnings.push(HdrtWarning::with_hint(
                "smartctl-failed",
                format!("smartctl failed for {path}: {err}"),
                "Run sudo hdrt disk --detail smart for more complete SMART information.",
            )),
        }
    }
}

fn apply_smartctl_output(disk: &mut DiskInfo, output: &str) {
    for line in output.lines() {
        if let Some(value) = line.strip_prefix("Model Family:") {
            disk.brand = non_empty_or_unknown(value.trim());
            disk.source = "lsblk + smartctl".to_string();
        } else if let Some(value) = line.strip_prefix("Firmware Version:") {
            disk.firmware = non_empty_or_unknown(value.trim());
        } else if let Some(value) =
            line.strip_prefix("SMART overall-health self-assessment test result:")
        {
            disk.health = non_empty_or_unknown(value.trim());
        }
    }
}
