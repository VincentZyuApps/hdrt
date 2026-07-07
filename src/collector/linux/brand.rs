use crate::hardware::{is_unknown, unknown};

pub(super) fn brand_from_vendor_or_model(vendor: Option<&str>, model: &str) -> String {
    vendor
        .and_then(normalize_brand)
        .or_else(|| infer_brand_from_model(model))
        .unwrap_or_else(unknown)
}

pub(super) fn brand_from_model_family(value: &str) -> Option<String> {
    let value = value.trim();
    if value.starts_with("Western Digital") {
        Some("Western Digital".to_string())
    } else if value.starts_with("Seagate") {
        Some("Seagate".to_string())
    } else if value.starts_with("Samsung") {
        Some("Samsung".to_string())
    } else if value.starts_with("TOSHIBA") || value.starts_with("Toshiba") {
        Some("Toshiba".to_string())
    } else if value.starts_with("HGST") {
        Some("HGST".to_string())
    } else if value.starts_with("Hitachi") {
        Some("Hitachi".to_string())
    } else {
        infer_brand_from_model(value)
    }
}

pub(super) fn infer_brand_from_model(model: &str) -> Option<String> {
    let model = model.trim();
    if model.is_empty() || is_unknown(model) {
        return None;
    }

    let upper = model.to_ascii_uppercase();
    if upper.starts_with("WDC ") || upper.starts_with("WD ") || upper.starts_with("WESTERN ") {
        return Some("Western Digital".to_string());
    }
    if upper.starts_with("ST") && upper.chars().nth(2).is_some_and(|ch| ch.is_ascii_digit()) {
        return Some("Seagate".to_string());
    }

    let first = model.split_whitespace().next()?;
    normalize_brand(first)
}

fn normalize_brand(value: &str) -> Option<String> {
    let value = value.trim().trim_matches(|ch: char| ch == '_' || ch == '-');
    if value.is_empty() || is_unknown(value) {
        return None;
    }

    match value.to_ascii_uppercase().as_str() {
        "ATA" | "NVME" | "USB" | "SCSI" | "SAS" | "SATA" | "GENERIC" | "LINUX" => None,
        "WDC" | "WD" => Some("Western Digital".to_string()),
        "TOSHIBA" => Some("Toshiba".to_string()),
        _ => Some(value.to_string()),
    }
}
