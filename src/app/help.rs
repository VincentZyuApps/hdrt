use std::ffi::OsString;
use std::path::Path;

use crate::i18n::Lang;

pub fn try_print_localized_help() -> bool {
    let args = std::env::args_os().collect::<Vec<_>>();
    let Some(request) = HelpRequest::from_args(&args) else {
        return false;
    };

    if !request.should_use_custom_help() {
        return false;
    }

    println!("{}", render_help(&request));
    true
}

#[derive(Debug)]
struct HelpRequest {
    bin_name: String,
    help_requested: bool,
    lang: Lang,
    lang_valid: bool,
    emoji: bool,
    topic: Option<HelpCommand>,
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
enum HelpCommand {
    Disk,
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

    fn name(self) -> &'static str {
        match self {
            Self::Disk => "disk",
            Self::Memory => "memory",
            Self::Cpu => "cpu",
            Self::Motherboard => "motherboard",
            Self::All => "all",
            Self::Doctor => "doctor",
            Self::Bench => "bench",
            Self::Tui => "tui",
        }
    }

    fn aliases(self) -> &'static str {
        match self {
            Self::Disk => "d",
            Self::Memory => "m, mem",
            Self::Cpu => "c",
            Self::Motherboard => "b, mb",
            Self::All => "a",
            Self::Doctor | Self::Bench | Self::Tui => "",
        }
    }

    fn icon(self) -> &'static str {
        match self {
            Self::Disk => "💽",
            Self::Memory => "🧠",
            Self::Cpu => "🧩",
            Self::Motherboard => "🧱",
            Self::All => "🖥️",
            Self::Doctor => "🩺",
            Self::Bench => "🧪",
            Self::Tui => "🖥️",
        }
    }

    fn description(self, lang: Lang) -> &'static str {
        match lang {
            Lang::EnUs => match self {
                Self::Disk => "Show physical disk information.",
                Self::Memory => "Show memory module information.",
                Self::Cpu => "Show CPU information.",
                Self::Motherboard => "Show motherboard and BIOS information.",
                Self::All => "Show all supported hardware sections.",
                Self::Doctor => "Show dependency, privilege, and backend status.",
                Self::Bench => "Benchmark available collection backends.",
                Self::Tui => "Open the Ratatui interface.",
            },
            Lang::ZhCn => match self {
                Self::Disk => "显示物理磁盘信息。",
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

const COMMANDS: &[HelpCommand] = &[
    HelpCommand::Disk,
    HelpCommand::Memory,
    HelpCommand::Cpu,
    HelpCommand::Motherboard,
    HelpCommand::All,
    HelpCommand::Doctor,
    HelpCommand::Bench,
    HelpCommand::Tui,
];

fn render_help(request: &HelpRequest) -> String {
    match request.topic {
        Some(command) => render_command_help(request, command),
        None => render_root_help(request),
    }
}

fn render_root_help(request: &HelpRequest) -> String {
    let lang = request.lang;
    let emoji = request.emoji;
    let mut lines = vec![
        title(lang, emoji),
        String::new(),
        format!(
            "{}: {} {}",
            heading(lang, emoji, HelpHeading::Usage),
            request.bin_name,
            usage_tail(lang, None)
        ),
        String::new(),
        format!("{}:", heading(lang, emoji, HelpHeading::Commands)),
    ];

    for command in COMMANDS {
        push_command(&mut lines, *command, lang, emoji);
    }
    push_help_command(&mut lines, lang, emoji);

    lines.push(String::new());
    lines.push(format!("{}:", heading(lang, emoji, HelpHeading::Options)));
    push_global_options(&mut lines, lang, emoji);
    lines.push(memory_hint(lang, emoji));

    lines.join("\n")
}

fn render_command_help(request: &HelpRequest, command: HelpCommand) -> String {
    let lang = request.lang;
    let emoji = request.emoji;
    let mut lines = vec![
        command_title(command, lang, emoji),
        String::new(),
        format!(
            "{}: {} {} {}",
            heading(lang, emoji, HelpHeading::Usage),
            request.bin_name,
            command.name(),
            usage_tail(lang, Some(command))
        ),
        String::new(),
        format!("{}:", heading(lang, emoji, HelpHeading::Options)),
    ];

    if matches!(command, HelpCommand::Tui) {
        push_tui_options(&mut lines, lang, emoji);
    }
    push_global_options(&mut lines, lang, emoji);
    lines.push(memory_hint(lang, emoji));

    lines.join("\n")
}

fn title(lang: Lang, emoji: bool) -> String {
    let text = match lang {
        Lang::EnUs => "Hardware Device Rust Ratatui: cross-platform hardware info CLI/TUI",
        Lang::ZhCn => "Hardware Device Rust Ratatui：跨平台硬件信息 CLI/TUI",
    };
    icon_text(emoji, "🖥️", text)
}

fn command_title(command: HelpCommand, lang: Lang, emoji: bool) -> String {
    icon_text(emoji, command.icon(), command.description(lang))
}

enum HelpHeading {
    Usage,
    Commands,
    Options,
}

fn heading(lang: Lang, emoji: bool, heading: HelpHeading) -> String {
    let (icon, text) = match (lang, heading) {
        (Lang::EnUs, HelpHeading::Usage) => ("🧭", "Usage"),
        (Lang::EnUs, HelpHeading::Commands) => ("📚", "Commands"),
        (Lang::EnUs, HelpHeading::Options) => ("⚙️", "Options"),
        (Lang::ZhCn, HelpHeading::Usage) => ("🧭", "用法"),
        (Lang::ZhCn, HelpHeading::Commands) => ("📚", "命令"),
        (Lang::ZhCn, HelpHeading::Options) => ("⚙️", "选项"),
    };
    icon_text(emoji, icon, text)
}

fn usage_tail(lang: Lang, command: Option<HelpCommand>) -> &'static str {
    match (lang, command) {
        (Lang::EnUs, None) => "[OPTIONS] [COMMAND]",
        (Lang::EnUs, Some(_)) => "[OPTIONS]",
        (Lang::ZhCn, None) => "[选项] [命令]",
        (Lang::ZhCn, Some(_)) => "[选项]",
    }
}

fn push_command(lines: &mut Vec<String>, command: HelpCommand, lang: Lang, emoji: bool) {
    let aliases = command.aliases();
    let alias = if aliases.is_empty() {
        String::new()
    } else {
        match lang {
            Lang::EnUs => format!(" [aliases: {aliases}]"),
            Lang::ZhCn => format!(" [别名: {aliases}]"),
        }
    };
    let description = command.description(lang);

    if emoji {
        lines.push(format!(
            "  {} {} - {}{}",
            command.icon(),
            command.name(),
            description,
            alias
        ));
    } else {
        lines.push(format!("  {:<12} {}{}", command.name(), description, alias));
    }
}

fn push_help_command(lines: &mut Vec<String>, lang: Lang, emoji: bool) {
    let description = match lang {
        Lang::EnUs => "Print this message or the help of the given subcommand(s).",
        Lang::ZhCn => "打印本帮助，或打印指定子命令的帮助。",
    };

    if emoji {
        lines.push(format!("  ❔ help - {description}"));
    } else {
        lines.push(format!("  {:<12} {}", "help", description));
    }
}

fn push_global_options(lines: &mut Vec<String>, lang: Lang, emoji: bool) {
    match lang {
        Lang::EnUs => {
            push_option(
                lines,
                emoji,
                "📋",
                "--format <FORMAT>",
                "Output format.",
                &["[default: table]", "[possible values: table, wide, compact, json, markdown]"],
            );
            push_option(
                lines,
                emoji,
                "🔎",
                "--detail <DETAIL>",
                "Hardware detail level.",
                &["[default: basic]", "[possible values: basic, smart, full]"],
            );
            push_option(
                lines,
                emoji,
                "🧩",
                "--backend <BACKEND>",
                "Hardware collection backend.",
                &[
                    "auto uses native collectors first and may use shell commands to fill missing fields.",
                    "[default: auto]",
                    "[possible values: auto, native, shell]",
                ],
            );
            push_option(
                lines,
                emoji,
                "⏳",
                "--no-spinner",
                "Disable the interactive loading spinner.",
                &[],
            );
            push_option(
                lines,
                emoji,
                "🌀",
                "--spinner-style <SPINNER_STYLE>",
                "Loading spinner style.",
                &["[default: unicode]", "[possible values: unicode, ascii, dots]"],
            );
            push_option(
                lines,
                emoji,
                "✨",
                "-e, --emoji",
                "Enable emoji decorations in CLI output, help, and TUI.",
                &[],
            );
            push_option(
                lines,
                emoji,
                "🌐",
                "--lang <LANG>",
                "Display language for help, table, markdown, and TUI output.",
                &["[default: en-us]", "[possible values: en-us, zh-cn]"],
            );
            push_option(
                lines,
                emoji,
                "🧪",
                "--debug",
                "Print additional collector diagnostics after normal output.",
                &[],
            );
            push_option(lines, emoji, "❔", "-h, --help", "Print help.", &[]);
            push_option(lines, emoji, "🏷️", "-V, --version", "Print version.", &[]);
        }
        Lang::ZhCn => {
            push_option(
                lines,
                emoji,
                "📋",
                "--format <FORMAT>",
                "输出格式。",
                &["[默认: table]", "[可选值: table, wide, compact, json, markdown]"],
            );
            push_option(
                lines,
                emoji,
                "🔎",
                "--detail <DETAIL>",
                "硬件信息详细程度。",
                &["[默认: basic]", "[可选值: basic, smart, full]"],
            );
            push_option(
                lines,
                emoji,
                "🧩",
                "--backend <BACKEND>",
                "硬件采集后端。",
                &[
                    "auto 会优先使用 native 采集器，并在需要时用 shell 命令补齐字段。",
                    "[默认: auto]",
                    "[可选值: auto, native, shell]",
                ],
            );
            push_option(lines, emoji, "⏳", "--no-spinner", "禁用交互式加载动画。", &[]);
            push_option(
                lines,
                emoji,
                "🌀",
                "--spinner-style <SPINNER_STYLE>",
                "加载动画样式。",
                &["[默认: unicode]", "[可选值: unicode, ascii, dots]"],
            );
            push_option(
                lines,
                emoji,
                "✨",
                "-e, --emoji",
                "启用 CLI 输出、帮助和 TUI 的 emoji 装饰。",
                &[],
            );
            push_option(
                lines,
                emoji,
                "🌐",
                "--lang <LANG>",
                "显示语言，用于帮助、表格、Markdown 和 TUI 输出。",
                &["[默认: en-us]", "[可选值: en-us, zh-cn]"],
            );
            push_option(
                lines,
                emoji,
                "🧪",
                "--debug",
                "在常规输出后打印额外采集诊断信息。",
                &[],
            );
            push_option(lines, emoji, "❔", "-h, --help", "打印帮助信息。", &[]);
            push_option(lines, emoji, "🏷️", "-V, --version", "打印版本。", &[]);
        }
    }
}

fn push_tui_options(lines: &mut Vec<String>, lang: Lang, emoji: bool) {
    match lang {
        Lang::EnUs => push_option(
            lines,
            emoji,
            "🗂️",
            "--tab <TAB>",
            "Initial TUI tab.",
            &[
                "[default: overview]",
                "[possible values: overview, disk, memory, cpu, motherboard, health, warnings]",
            ],
        ),
        Lang::ZhCn => push_option(
            lines,
            emoji,
            "🗂️",
            "--tab <TAB>",
            "TUI 初始标签页。",
            &[
                "[默认: overview]",
                "[可选值: overview, disk, memory, cpu, motherboard, health, warnings]",
            ],
        ),
    }
}

fn push_option(
    lines: &mut Vec<String>,
    emoji: bool,
    icon: &str,
    syntax: &str,
    description: &str,
    details: &[&str],
) {
    if emoji {
        lines.push(format!("  {icon} {syntax}"));
    } else {
        lines.push(format!("      {syntax}"));
    }
    lines.push(format!("          {description}"));
    for detail in details {
        lines.push(format!("          {detail}"));
    }
    lines.push(String::new());
}

fn memory_hint(lang: Lang, emoji: bool) -> String {
    let text = match lang {
        Lang::EnUs => "Memory hint: hdrt can be remembered as \"hard rata\".",
        Lang::ZhCn => "记忆提示：hdrt 可以记作 \"hard rata\"。",
    };
    icon_text(emoji, "💡", text)
}

fn icon_text(emoji: bool, icon: &str, text: &str) -> String {
    if emoji {
        format!("{icon} {text}")
    } else {
        text.to_string()
    }
}

fn option_takes_value(token: &str) -> bool {
    matches!(
        token,
        "--format" | "--detail" | "--backend" | "--spinner-style" | "--tab"
    )
}

fn bin_name(value: &OsString) -> String {
    Path::new(value)
        .file_name()
        .and_then(|value| value.to_str())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| "hdrt".to_string())
}
