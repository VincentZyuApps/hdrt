use crate::hardware::{is_unknown, unknown};

const IGNORED_VENDOR_TOKENS: &[&str] = &[
    "ATA",
    "NVME",
    "NVM EXPRESS",
    "USB",
    "SCSI",
    "SAS",
    "SATA",
    "GENERIC",
    "LINUX",
    "(STANDARD DISK DRIVES)",
];

const KNOWN_BRANDS: &[(&str, &str)] = &[
    ("GREAT WALL", "Great Wall"),
    ("WESTERN DIGITAL", "Western Digital"),
    ("SANDISK", "SanDisk"),
    ("KINGSTON", "Kingston"),
    ("COLORFUL", "Colorful"),
    ("REALTEK", "Realtek"),
    ("QUANXING", "QUANXING"),
    ("SEAGATE", "Seagate"),
    ("SAMSUNG", "Samsung"),
    ("TOSHIBA", "Toshiba"),
    ("HITACHI", "Hitachi"),
    ("HGST", "HGST"),
    ("INTEL", "Intel"),
    ("CRUCIAL", "Crucial"),
    ("MICRON", "Micron"),
    ("SK HYNIX", "SK hynix"),
];

pub(crate) fn brand_from_vendor_or_model(vendor: Option<&str>, model: &str) -> String {
    vendor
        .and_then(normalize_brand)
        .or_else(|| infer_brand_from_model(model))
        .unwrap_or_else(unknown)
}

pub(crate) fn brand_from_vendor_candidates<'a>(
    vendors: impl IntoIterator<Item = Option<&'a str>>,
    model: &str,
) -> String {
    vendors
        .into_iter()
        .flatten()
        .find_map(normalize_brand)
        .or_else(|| infer_brand_from_model(model))
        .unwrap_or_else(unknown)
}

pub(crate) fn brand_from_model_family(value: &str) -> Option<String> {
    known_brand_prefix(value).or_else(|| infer_brand_from_model(value))
}

pub(crate) fn infer_brand_from_model(model: &str) -> Option<String> {
    let model = model.trim();
    if model.is_empty() || is_unknown(model) {
        return None;
    }

    known_brand_prefix(model).or_else(|| {
        let upper = model.to_ascii_uppercase();
        if upper.starts_with("WDC ") || upper.starts_with("WD ") || upper.starts_with("WD_") {
            return Some("Western Digital".to_string());
        }
        if upper.starts_with("ST") && upper.chars().nth(2).is_some_and(|ch| ch.is_ascii_digit()) {
            return Some("Seagate".to_string());
        }

        let first = model.split_whitespace().next()?;
        normalize_brand(first)
    })
}

fn normalize_brand(value: &str) -> Option<String> {
    let value = clean_brand_candidate(value);
    if value.is_empty() || is_unknown(&value) {
        return None;
    }

    let upper = value.to_ascii_uppercase();
    if IGNORED_VENDOR_TOKENS.iter().any(|token| *token == upper) {
        return None;
    }

    known_brand_prefix(&value).or_else(|| match upper.as_str() {
        "WDC" | "WD" => Some("Western Digital".to_string()),
        _ => Some(value),
    })
}

fn known_brand_prefix(value: &str) -> Option<String> {
    let upper = value.to_ascii_uppercase();
    KNOWN_BRANDS
        .iter()
        .find(|(prefix, _)| upper == *prefix || upper.starts_with(&format!("{prefix} ")))
        .map(|(_, brand)| (*brand).to_string())
}

fn clean_brand_candidate(value: &str) -> String {
    value
        .trim()
        .trim_matches(|ch: char| ch == '_' || ch == '-' || ch == '\0')
        .replace('_', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn infers_multi_word_brand() {
        assert_eq!(
            infer_brand_from_model("Great Wall GW560 512GB"),
            Some("Great Wall".to_string())
        );
    }

    #[test]
    fn ignores_transport_vendor_tokens() {
        assert_eq!(
            brand_from_vendor_or_model(Some("ATA"), "ST500DM002-1SB10A"),
            "Seagate"
        );
    }
}
