use std::process::Command;

use crate::app::options::DetailLevel;
use crate::hardware::{is_unknown, unknown, DiskInfo, HdrtWarning};

use super::brand::{brand_from_model_family, brand_from_vendor_or_model, infer_brand_from_model};
use super::command::{
    format_bytes, non_empty_or_unknown, parse_key_values, run_command, run_shell_script,
    value_or_unknown,
};

pub(super) fn collect(detail: DetailLevel) -> Vec<DiskInfo> {
    let output = run_shell_script(include_str!("scripts/collect_disks.sh"));

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
            let bus = value_or_unknown(values.get("TRAN"));
            let model = value_or_unknown(values.get("MODEL"));

            Some(DiskInfo {
                device: name,
                model: model.clone(),
                brand: brand_from_vendor_or_model(values.get("VENDOR").map(String::as_str), &model),
                serial: value_or_unknown(values.get("SERIAL")),
                size: value_or_unknown(values.get("SIZE")),
                media_type: media_type(rota, &bus),
                bus,
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

    enrich_with_smartctl(&mut disks, detail);

    disks
}

fn media_type(rota: &str, bus: &str) -> String {
    if bus.eq_ignore_ascii_case("nvme") {
        return "NVMe SSD".to_string();
    }

    match rota {
        "0" => "SSD".to_string(),
        "1" => "HDD".to_string(),
        _ => unknown(),
    }
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

fn enrich_with_smartctl(disks: &mut [DiskInfo], detail: DetailLevel) {
    let require_smart = matches!(detail, DetailLevel::Smart | DetailLevel::Full);

    if which::which("smartctl").is_err() {
        if !require_smart {
            return;
        }

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
        let args = if require_smart {
            vec!["-a", path.as_str()]
        } else {
            vec!["-i", "-H", path.as_str()]
        };

        match run_smartctl(&args) {
            Ok(output) => apply_smartctl_output(disk, &output),
            Err(err) => {
                if require_smart {
                    disk.warnings.push(HdrtWarning::with_hint(
                        "smartctl-failed",
                        format!("smartctl failed for {path}: {err}"),
                        "Run sudo hdrt disk --detail smart for more complete SMART information.",
                    ));
                }
            }
        }
    }
}

fn run_smartctl(args: &[&str]) -> Result<String, String> {
    let output = Command::new("smartctl")
        .env("LC_ALL", "C")
        .args(args)
        .output()
        .map_err(|err| err.to_string())?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    if !stdout.trim().is_empty() {
        return Ok(stdout);
    }

    if output.status.success() {
        Ok(stdout)
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

fn apply_smartctl_output(disk: &mut DiskInfo, output: &str) {
    for line in output.lines() {
        if let Some(value) = line.strip_prefix("Model Family:") {
            if let Some(brand) = brand_from_model_family(value) {
                disk.brand = brand;
            }
            disk.source = "lsblk + smartctl".to_string();
        } else if let Some(value) = line.strip_prefix("Device Model:") {
            apply_model_value(disk, value);
        } else if let Some(value) = line.strip_prefix("Model Number:") {
            apply_model_value(disk, value);
        } else if let Some(value) = line.strip_prefix("Serial Number:") {
            if is_unknown(&disk.serial) {
                disk.serial = non_empty_or_unknown(value.trim());
            }
        } else if let Some(value) = line.strip_prefix("Firmware Version:") {
            disk.firmware = non_empty_or_unknown(value.trim());
        } else if let Some(value) =
            line.strip_prefix("SMART overall-health self-assessment test result:")
        {
            disk.health = normalize_health(value);
        } else if let Some(value) = line.strip_prefix("SMART Health Status:") {
            disk.health = normalize_health(value);
        }
    }
}

fn apply_model_value(disk: &mut DiskInfo, value: &str) {
    let model = non_empty_or_unknown(value.trim());
    if is_unknown(&disk.model) {
        disk.model = model.clone();
    }
    if is_unknown(&disk.brand) {
        disk.brand = infer_brand_from_model(&model).unwrap_or_else(unknown);
    }
}

fn normalize_health(value: &str) -> String {
    let value = value.trim();
    let upper = value.to_ascii_uppercase();
    if upper.contains("PASSED") || upper == "OK" {
        "Healthy".to_string()
    } else if upper.contains("FAILED") || upper.contains("FAIL") {
        "Unhealthy".to_string()
    } else if upper.contains("WARN") {
        "Warning".to_string()
    } else {
        non_empty_or_unknown(value)
    }
}
