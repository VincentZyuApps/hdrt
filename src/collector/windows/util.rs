use serde_json::Value;

pub fn value_array<'a>(root: &'a Value, key: &str) -> Vec<&'a Value> {
    match root.get(key) {
        Some(Value::Array(values)) => values.iter().collect(),
        Some(Value::Null) | None => Vec::new(),
        Some(value) => vec![value],
    }
}

pub fn value_string(value: &Value, key: &str) -> String {
    match value.get(key) {
        Some(Value::String(text)) => known_or_unknown(text.trim()),
        Some(Value::Number(number)) => number.to_string(),
        Some(Value::Bool(boolean)) => boolean.to_string(),
        _ => "Unknown".to_string(),
    }
}

pub fn value_u64(value: &Value, key: &str) -> Option<u64> {
    match value.get(key) {
        Some(Value::Number(number)) => number.as_u64(),
        Some(Value::String(text)) => text.trim().parse().ok(),
        _ => None,
    }
}

pub fn first_known(values: &[String]) -> String {
    values
        .iter()
        .find(|value| value.as_str() != "Unknown")
        .cloned()
        .unwrap_or_else(|| "Unknown".to_string())
}

pub fn clean_manufacturer(value: &str) -> String {
    if value == "(Standard disk drives)" || value.trim().is_empty() {
        "Unknown".to_string()
    } else {
        value.trim().to_string()
    }
}

pub fn format_bytes(bytes: u64) -> String {
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

fn known_or_unknown(value: &str) -> String {
    if value.is_empty() {
        "Unknown".to_string()
    } else {
        value.to_string()
    }
}
