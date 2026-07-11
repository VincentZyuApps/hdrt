use std::ffi::OsString;
use std::path::Path;

use crate::i18n::Lang;

mod render;

pub fn try_print_localized_help() -> bool {
    let args = std::env::args_os().collect::<Vec<_>>();
    let Some(request) = HelpRequest::from_args(&args) else {
        return false;
    };

    if !request.should_use_custom_help() {
        return false;
    }

    println!("{}", render::render_help(&request));
    true
}

#[derive(Debug)]
pub(super) struct HelpRequest {
    pub(super) bin_name: String,
    help_requested: bool,
    pub(super) lang: Lang,
    lang_valid: bool,
    pub(super) emoji: bool,
    pub(super) topic: Option<HelpCommand>,
}

impl HelpRequest {
    fn from_args(args: &[OsString]) -> Option<Self> {
        let bin_name = args
            .first()
            .map(bin_name)
            .unwrap_or_else(|| "hdrt".to_string());

        let mut request = Self {
            bin_name,
            help_requested: false,
            lang: Lang::EnUs,
            lang_valid: true,
            emoji: false,
            topic: None,
        };

        let tokens = args
            .iter()
            .skip(1)
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        let mut index = 0;

        while index < tokens.len() {
            let token = tokens[index].as_str();

            if token == "help" {
                request.help_requested = true;
                index += 1;
                continue;
            }

            if token == "--help" {
                request.help_requested = true;
                index += 1;
                continue;
            }

            if token.starts_with("--lang=") {
                request.set_lang(token.trim_start_matches("--lang="));
                index += 1;
                continue;
            }

            if token == "--lang" {
                if let Some(value) = tokens.get(index + 1) {
                    request.set_lang(value);
                    index += 2;
                } else {
                    request.lang_valid = false;
                    index += 1;
                }
                continue;
            }

            if token == "--emoji" {
                request.emoji = true;
                index += 1;
                continue;
            }

            if token.starts_with("--") {
                index += if option_takes_value(token) { 2 } else { 1 };
                continue;
            }

            if token.starts_with('-') && token.len() > 1 {
                for flag in token.chars().skip(1) {
                    match flag {
                        'h' => request.help_requested = true,
                        'e' => request.emoji = true,
                        _ => {}
                    }
                }
                index += 1;
                continue;
            }

            if request.topic.is_none() {
                request.topic = HelpCommand::from_arg(token);
            }

            index += 1;
        }

        Some(request)
    }

    fn set_lang(&mut self, value: &str) {
        match value {
            "en-us" => self.lang = Lang::EnUs,
            "zh-cn" => self.lang = Lang::ZhCn,
            _ => self.lang_valid = false,
        }
    }

    fn should_use_custom_help(&self) -> bool {
        self.help_requested && self.lang_valid && (self.lang == Lang::ZhCn || self.emoji)
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) enum HelpCommand {
    Disk,
    PhysicalDisk,
    LogicalDisk,
    Memory,
    Cpu,
    Motherboard,
    All,
    Doctor,
    Bench,
    Tui,
}

impl HelpCommand {
    fn from_arg(value: &str) -> Option<Self> {
        match value {
            "disk" | "d" => Some(Self::Disk),
            "physical-disk" | "pd" => Some(Self::PhysicalDisk),
            "logical-disk" | "ld" => Some(Self::LogicalDisk),
            "memory" | "m" | "mem" => Some(Self::Memory),
            "cpu" | "c" => Some(Self::Cpu),
            "motherboard" | "b" | "mb" => Some(Self::Motherboard),
            "all" | "a" => Some(Self::All),
            "doctor" => Some(Self::Doctor),
            "bench" => Some(Self::Bench),
            "tui" => Some(Self::Tui),
            _ => None,
        }
    }

    pub(super) fn name(self) -> &'static str {
        match self {
            Self::Disk => "disk",
            Self::PhysicalDisk => "physical-disk",
            Self::LogicalDisk => "logical-disk",
            Self::Memory => "memory",
            Self::Cpu => "cpu",
            Self::Motherboard => "motherboard",
            Self::All => "all",
            Self::Doctor => "doctor",
            Self::Bench => "bench",
            Self::Tui => "tui",
        }
    }

    pub(super) fn aliases(self) -> &'static str {
        match self {
            Self::Disk => "d",
            Self::PhysicalDisk => "pd",
            Self::LogicalDisk => "ld",
            Self::Memory => "m, mem",
            Self::Cpu => "c",
            Self::Motherboard => "b, mb",
            Self::All => "a",
            Self::Doctor | Self::Bench | Self::Tui => "",
        }
    }

    pub(super) fn icon(self) -> &'static str {
        match self {
            Self::Disk => "💽",
            Self::PhysicalDisk => "💽",
            Self::LogicalDisk => "🗂️",
            Self::Memory => "🧠",
            Self::Cpu => "🧩",
            Self::Motherboard => "🧱",
            Self::All => "🖥️",
            Self::Doctor => "🩺",
            Self::Bench => "🧪",
            Self::Tui => "🖥️",
        }
    }

    pub(super) fn description(self, lang: Lang) -> &'static str {
        match lang {
            Lang::EnUs => match self {
                Self::Disk => "Show physical and logical disk information.",
                Self::PhysicalDisk => "Show physical disk information.",
                Self::LogicalDisk => "Show logical disk information.",
                Self::Memory => "Show memory module information.",
                Self::Cpu => "Show CPU information.",
                Self::Motherboard => "Show motherboard and BIOS information.",
                Self::All => "Show all supported hardware sections.",
                Self::Doctor => "Show dependency, privilege, and backend status.",
                Self::Bench => "Benchmark available collection backends.",
                Self::Tui => "Open the Ratatui interface.",
            },
            Lang::ZhCn => match self {
                Self::Disk => "同时显示物理磁盘和逻辑磁盘。",
                Self::PhysicalDisk => "显示物理磁盘信息。",
                Self::LogicalDisk => "显示逻辑磁盘信息。",
                Self::Memory => "显示内存条信息。",
                Self::Cpu => "显示 CPU 信息。",
                Self::Motherboard => "显示主板和 BIOS 信息。",
                Self::All => "显示所有已支持的硬件分区。",
                Self::Doctor => "显示依赖、权限和后端状态。",
                Self::Bench => "测试可用采集后端。",
                Self::Tui => "打开 Ratatui 交互界面。",
            },
        }
    }
}

fn option_takes_value(token: &str) -> bool {
    matches!(
        token,
        "--format"
            | "--style"
            | "--table-style"
            | "--detail"
            | "--backend"
            | "--spinner-style"
            | "--tab"
            | "--chart-mode"
            | "--border"
            | "--tui-border"
            | "--interval"
    )
}

fn bin_name(value: &OsString) -> String {
    Path::new(value)
        .file_name()
        .and_then(|value| value.to_str())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| "hdrt".to_string())
}
