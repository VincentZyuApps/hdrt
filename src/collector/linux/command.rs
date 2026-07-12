use std::collections::HashMap;
use std::process::Command;

use crate::hardware::unknown;

pub(super) fn run_shell_script(script: &str) -> Result<String, String> {
    run_shell_script_with_args(script, &[])
}

pub(super) fn run_shell_script_with_args(script: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(script)
        .arg("hdrt-script")
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

pub(super) fn parse_key_values(line: &str) -> HashMap<String, String> {
    let mut values = HashMap::new();
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        while i < chars.len() && chars[i].is_whitespace() {
            i += 1;
        }

        let key_start = i;
        while i < chars.len() && chars[i] != '=' {
            i += 1;
        }
        if i >= chars.len() {
            break;
        }
        let key: String = chars[key_start..i].iter().collect();
        i += 1;

        if i >= chars.len() || chars[i] != '"' {
            break;
        }
        i += 1;
        let value_start = i;
        while i < chars.len() && chars[i] != '"' {
            i += 1;
        }
        let value: String = chars[value_start..i].iter().collect();
        if i < chars.len() {
            i += 1;
        }
        values.insert(key, value);
    }

    values
}

pub(super) fn field_value<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let (line_key, value) = line.split_once(':')?;
    if line_key.trim() == key {
        Some(value.trim())
    } else {
        None
    }
}

pub(super) fn value_or_unknown(value: Option<&String>) -> String {
    value
        .map(|value| non_empty_or_unknown(value.trim()))
        .unwrap_or_else(unknown)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_lsblk_key_value_rows_with_spaces() {
        let values = parse_key_values(
            "NAME=\"sda\" MODEL=\"Fixture SSD 1TB\" SERIAL=\"ABC123\" TRAN=\"sata\"",
        );

        assert_eq!(values["NAME"], "sda");
        assert_eq!(values["MODEL"], "Fixture SSD 1TB");
        assert_eq!(values["SERIAL"], "ABC123");
    }
}
