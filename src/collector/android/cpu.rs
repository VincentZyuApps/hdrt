use std::collections::HashMap;
use std::fs;

use crate::hardware::{unknown, CpuInfo};

use super::command::{field_value, parse_getprop, run_command};

pub(super) fn collect() -> Option<CpuInfo> {
    let text = fs::read_to_string("/proc/cpuinfo").unwrap_or_default();
    let props = run_command("getprop", &[])
        .map(|output| parse_getprop(&output))
        .unwrap_or_default();
    let mut hardware = None;
    let mut processor = None;
    let mut model_name = None;
    let mut vendor_id = None;
    let mut implementer = None;
    let mut cpuinfo_frequency = None;
    let mut physical_cores = None;
    let mut logical_threads = 0usize;

    for line in text.lines() {
        if let Some(value) = field_value(line, "Hardware") {
            hardware.get_or_insert_with(|| value.to_string());
        } else if let Some(value) = field_value(line, "Processor") {
            processor.get_or_insert_with(|| value.to_string());
        } else if let Some(value) = field_value(line, "model name") {
            model_name.get_or_insert_with(|| value.to_string());
        } else if let Some(value) = field_value(line, "vendor_id") {
            vendor_id.get_or_insert_with(|| value.to_string());
        } else if let Some(value) = field_value(line, "CPU implementer") {
            implementer.get_or_insert_with(|| value.to_string());
        } else if let Some(value) = field_value(line, "cpu MHz") {
            cpuinfo_frequency.get_or_insert_with(|| format!("{value} MHz"));
        } else if field_value(line, "processor").is_some() {
            logical_threads += 1;
        } else if let Some(value) = field_value(line, "cpu cores") {
            if physical_cores.is_none() {
                physical_cores = value.parse().ok();
            }
        }
    }

    let logical_threads = (logical_threads > 0)
        .then_some(logical_threads)
        .or_else(|| {
            std::thread::available_parallelism()
                .ok()
                .map(|value| value.get())
        });
    let model = first_value(&[
        prop(&props, "ro.soc.model"),
        prop(&props, "ro.board.platform"),
        hardware,
        model_name,
        processor,
        prop(&props, "ro.hardware"),
    ])
    .unwrap_or_else(unknown);
    let vendor = first_value(&[
        prop(&props, "ro.soc.manufacturer"),
        vendor_id,
        implementer.and_then(|value| cpu_implementer_name(&value).map(str::to_string)),
    ])
    .unwrap_or_else(unknown);
    let frequency = max_cpu_frequency()
        .or(cpuinfo_frequency)
        .unwrap_or_else(unknown);

    Some(CpuInfo {
        model,
        vendor,
        physical_cores,
        logical_threads,
        frequency,
        source: "/proc/cpuinfo + /sys/devices/system/cpu + getprop".to_string(),
        warnings: Vec::new(),
    })
}

fn prop(props: &HashMap<String, String>, key: &str) -> Option<String> {
    props
        .get(key)
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn first_value(values: &[Option<String>]) -> Option<String> {
    values
        .iter()
        .flatten()
        .find(|value| !value.trim().is_empty())
        .cloned()
}

fn max_cpu_frequency() -> Option<String> {
    let entries = fs::read_dir("/sys/devices/system/cpu").ok()?;
    let max_khz = entries
        .filter_map(Result::ok)
        .filter(|entry| is_cpu_directory(&entry.file_name().to_string_lossy()))
        .filter_map(|entry| {
            ["cpuinfo_max_freq", "scaling_max_freq"]
                .iter()
                .find_map(|name| {
                    fs::read_to_string(entry.path().join("cpufreq").join(name))
                        .ok()
                        .and_then(|value| value.trim().parse::<u64>().ok())
                })
        })
        .max()?;

    Some(format!("{} MHz", max_khz / 1_000))
}

fn is_cpu_directory(name: &str) -> bool {
    name.strip_prefix("cpu")
        .is_some_and(|suffix| !suffix.is_empty() && suffix.chars().all(|ch| ch.is_ascii_digit()))
}

fn cpu_implementer_name(value: &str) -> Option<&'static str> {
    match value.trim().to_ascii_lowercase().as_str() {
        "0x41" => Some("Arm"),
        "0x42" => Some("Broadcom"),
        "0x43" => Some("Cavium"),
        "0x46" => Some("Fujitsu"),
        "0x48" => Some("HiSilicon"),
        "0x4e" => Some("NVIDIA"),
        "0x51" => Some("Qualcomm"),
        "0x53" => Some("Samsung"),
        "0x56" => Some("Marvell"),
        "0x61" => Some("Apple"),
        "0x69" => Some("Intel"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_cpu_sysfs_directory_names() {
        assert!(is_cpu_directory("cpu0"));
        assert!(is_cpu_directory("cpu12"));
        assert!(!is_cpu_directory("cpufreq"));
        assert!(!is_cpu_directory("cpu"));
    }

    #[test]
    fn maps_common_arm_cpu_implementers() {
        assert_eq!(cpu_implementer_name("0x41"), Some("Arm"));
        assert_eq!(cpu_implementer_name("0x51"), Some("Qualcomm"));
        assert_eq!(cpu_implementer_name("0xff"), None);
    }

    #[test]
    fn first_value_skips_empty_candidates() {
        assert_eq!(
            first_value(&[Some(String::new()), Some("Tensor".to_string())]),
            Some("Tensor".to_string())
        );
    }
}
