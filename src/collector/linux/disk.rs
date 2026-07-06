use crate::app::options::DetailLevel;
use crate::hardware::{unknown, DiskInfo, HdrtWarning};

use super::command::{
    format_bytes, non_empty_or_unknown, parse_key_values, run_command, value_or_unknown,
};

pub(super) fn collect(detail: DetailLevel) -> Vec<DiskInfo> {
    let output = run_command(
        "lsblk",
        &[
            "-d",
            "-P",
            "-o",
            "NAME,MODEL,SERIAL,SIZE,ROTA,TYPE,TRAN,VENDOR,REV",
        ],
    );

    let Ok(output) = output else {
        return collect_df_logical_disks_with_warning(
            "lsblk-unavailable",
            "Could not run lsblk to collect physical disk inventory; using df logical volumes.",
            "Install util-linux for model, serial, bus, and firmware fields.",
        );
    };

    let mut disks: Vec<DiskInfo> = output
        .lines()
        .filter_map(|line| {
            let values = parse_key_values(line);
            let name = values.get("NAME")?.to_string();
            let rota = values.get("ROTA").map(String::as_str).unwrap_or("");
            let media_type = match rota {
                "0" => "SSD/NVMe".to_string(),
                "1" => "HDD".to_string(),
                _ => unknown(),
            };

            Some(DiskInfo {
                device: name,
                model: value_or_unknown(values.get("MODEL")),
                serial: value_or_unknown(values.get("SERIAL")),
                size: value_or_unknown(values.get("SIZE")),
                media_type,
                bus: value_or_unknown(values.get("TRAN")),
                firmware: value_or_unknown(values.get("REV")),
                source: "lsblk".to_string(),
                ..DiskInfo::default()
            })
        })
        .collect();

    if disks.is_empty() {
        return collect_df_logical_disks_with_warning(
            "lsblk-empty",
            "lsblk returned no parseable physical disk rows; using df logical volumes.",
            "Check lsblk output support with: lsblk -d -P -o NAME,MODEL,SERIAL,SIZE,ROTA,TYPE,TRAN,VENDOR,REV.",
        );
    }

    if matches!(detail, DetailLevel::Smart | DetailLevel::Full) {
        enrich_with_smartctl(&mut disks);
    }

    disks
}

fn collect_df_logical_disks() -> Vec<DiskInfo> {
    match run_command("df", &["-kP"]) {
        Ok(output) => parse_df_logical_disks(&output),
        Err(_) => Vec::new(),
    }
}

fn collect_df_logical_disks_with_warning(code: &str, message: &str, hint: &str) -> Vec<DiskInfo> {
    let mut disks = collect_df_logical_disks();
    if disks.is_empty() {
        return vec![DiskInfo {
            warnings: vec![HdrtWarning::with_hint(
                "linux-disk-inventory-unavailable",
                "Could not run lsblk or df to collect disk inventory.",
                "Install util-linux or coreutils and run hdrt again.",
            )],
            ..DiskInfo::default()
        }];
    }

    for disk in &mut disks {
        disk.warnings
            .push(HdrtWarning::with_hint(code, message, hint));
    }

    disks
}

fn parse_df_logical_disks(output: &str) -> Vec<DiskInfo> {
    output
        .lines()
        .skip(1)
        .filter_map(|line| {
            let fields: Vec<&str> = line.split_whitespace().collect();
            if fields.len() < 6 {
                return None;
            }

            let filesystem = fields[0];
            let blocks_kib = fields.get(1)?.parse::<u64>().ok()?;
            let mount = fields[5];

            Some(DiskInfo {
                device: mount.to_string(),
                model: filesystem.to_string(),
                size: format_bytes(blocks_kib * 1024),
                media_type: "Logical".to_string(),
                source: "df".to_string(),
                ..DiskInfo::default()
            })
        })
        .collect()
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
