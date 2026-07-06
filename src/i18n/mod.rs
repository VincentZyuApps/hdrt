mod en_us;
mod zh_cn;

use clap::ValueEnum;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Lang {
    #[value(name = "en-us")]
    EnUs,
    #[value(name = "zh-cn")]
    ZhCn,
}

impl fmt::Display for Lang {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Lang::EnUs => "en-us",
            Lang::ZhCn => "zh-cn",
        })
    }
}

pub fn t(lang: Lang, key: &str) -> &'static str {
    match lang {
        Lang::EnUs => en_us::t(key),
        Lang::ZhCn => zh_cn::t(key),
    }
}

pub fn display_value(lang: Lang, value: &str) -> String {
    if is_unknown_display_value(value) {
        t(lang, "unknown").to_string()
    } else {
        value.to_string()
    }
}

pub fn display_optional(lang: Lang, value: Option<impl ToString>) -> String {
    value
        .map(|value| display_value(lang, &value.to_string()))
        .unwrap_or_else(|| t(lang, "unknown").to_string())
}

fn is_unknown_display_value(value: &str) -> bool {
    let normalized = value.trim().trim_end_matches('.').to_ascii_lowercase();

    normalized.is_empty()
        || normalized == "unknown"
        || normalized == crate::hardware::UNKNOWN.to_ascii_lowercase()
        || normalized == "default string"
        || normalized == "to be filled by o.e.m"
        || normalized == "to be filled by oem"
        || normalized == "system serial number"
        || normalized == "none"
        || normalized == "not specified"
}
