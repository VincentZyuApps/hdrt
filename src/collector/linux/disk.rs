use crate::app::options::DetailLevel;
use crate::hardware::{is_unknown, unknown, DiskInfo, HdrtWarning};

use super::command::{
    non_empty_or_unknown, parse_key_values, run_shell_script, run_shell_script_with_args,
    value_or_unknown,
};

pub(super) fn collect(detail: DetailLevel) -> Vec<DiskInfo> {
    let output = run_shell_script(include_str!("scripts/collect_disks.sh"));

    let Ok(output) = output else {
        return unavailable_physical_disk(
            "lsblk-unavailable",
            "Could not run lsblk to collect physical disk inventory.",
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
        return unavailable_physical_disk(
            "lsblk-empty",
            "lsblk returned no parseable physical disk rows.",
            "Check lsblk output support with: lsblk -d -P -o NAME,MODEL,SERIAL,SIZE,ROTA,TYPE,TRAN,REV.",
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

fn unavailable_physical_disk(code: &str, message: &str, hint: &str) -> Vec<DiskInfo> {
    vec![DiskInfo {
        warnings: vec![HdrtWarning::with_hint(code, message, hint)],
        ..DiskInfo::default()
    }]
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
        let detail_arg = if require_smart {
            detail.into_script_arg()
        } else {
            "basic"
        };

        match run_smartctl(detail_arg, &path) {
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

fn run_smartctl(detail: &str, path: &str) -> Result<String, String> {
    run_shell_script_with_args(include_str!("scripts/collect_smart.sh"), &[detail, path])
}

trait DetailLevelScriptArg {
    fn into_script_arg(self) -> &'static str;
}

impl DetailLevelScriptArg for DetailLevel {
    fn into_script_arg(self) -> &'static str {
        match self {
            DetailLevel::Basic => "basic",
            DetailLevel::Smart => "smart",
            DetailLevel::Full => "full",
        }
    }
}

fn apply_smartctl_output(disk: &mut DiskInfo, output: &str) {
    for line in output.lines() {
        if line.strip_prefix("Model Family:").is_some() {
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
        disk.model = model;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_common_smart_health_values() {
        assert_eq!(normalize_health("PASSED"), "Healthy");
        assert_eq!(normalize_health("FAILED!"), "Unhealthy");
        assert_eq!(normalize_health("Warning"), "Warning");
    }

    #[test]
    fn smartctl_fills_missing_disk_details() {
        let mut disk = DiskInfo::default();
        apply_smartctl_output(
            &mut disk,
            "Device Model: Fixture SSD\nSerial Number: SERIAL-1\nFirmware Version: FW-1\nSMART overall-health self-assessment test result: PASSED\n",
        );

        assert_eq!(disk.model, "Fixture SSD");
        assert_eq!(disk.serial, "SERIAL-1");
        assert_eq!(disk.firmware, "FW-1");
        assert_eq!(disk.health, "Healthy");
    }
}
