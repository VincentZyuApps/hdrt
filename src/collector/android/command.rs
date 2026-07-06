use std::collections::HashMap;
use std::process::Command;

use crate::hardware::unknown;

pub(super) fn run_command(program: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|err| err.to_string())?;

    if output.status.success() {
        String::from_utf8(output.stdout).map_err(|err| err.to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(stderr.trim().to_string())
    }
}

pub(super) fn field_value<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let (line_key, value) = line.split_once(':')?;
    if line_key.trim() == key {
        Some(value.trim())
    } else {
        None
    }
}

pub(super) fn parse_getprop(output: &str) -> HashMap<String, String> {
    output
        .lines()
        .filter_map(|line| {
            let (key, value) = line.split_once("]: [")?;
            let key = key.strip_prefix('[')?;
            let value = value.strip_suffix(']')?;
            Some((key.to_string(), value.to_string()))
        })
        .collect()
}

pub(super) fn non_empty_or_unknown(value: &str) -> String {
    if value.trim().is_empty() {
        unknown()
    } else {
        value.trim().to_string()
    }
}

pub(super) fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} {}", UNITS[unit])
    } else {
        format!("{value:.2} {}", UNITS[unit])
    }
}
