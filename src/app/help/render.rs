use crate::i18n::Lang;

use super::{HelpCommand, HelpRequest};

const COMMANDS: &[HelpCommand] = &[
    HelpCommand::Disk,
    HelpCommand::PhysicalDisk,
    HelpCommand::LogicalDisk,
    HelpCommand::Memory,
    HelpCommand::Cpu,
    HelpCommand::Motherboard,
    HelpCommand::All,
    HelpCommand::Doctor,
    HelpCommand::Bench,
    HelpCommand::Tui,
];

pub(super) fn render_help(request: &HelpRequest) -> String {
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
                "CLI render format.",
                &[
                    "[default: table]",
                    "[possible values: table, json, markdown]",
                ],
            );
            push_option(
                lines,
                emoji,
                "🎨",
                "--style <STYLE>",
                "CLI table style. Alias: --table-style.",
                &[
                    "[default: rounded]",
                    "[possible values: rounded, modern, sharp, psql, ascii, blank]",
                    "[aliases: round -> rounded, plain -> ascii]",
                ],
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
                &[
                    "[default: unicode]",
                    "[possible values: unicode, ascii, dots]",
                ],
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
                "🎨",
                "--no-color",
                "Disable ANSI colors in CLI output and TUI chrome.",
                &[],
            );
            push_option(
                lines,
                emoji,
                "🔠",
                "--no-bold",
                "Disable ANSI bold text in CLI output and TUI chrome.",
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
                "CLI 渲染格式。",
                &["[默认: table]", "[可选值: table, json, markdown]"],
            );
            push_option(
                lines,
                emoji,
                "🎨",
                "--style <STYLE>",
                "CLI 表格样式。别名：--table-style。",
                &[
                    "[默认: rounded]",
                    "[可选值: rounded, modern, sharp, psql, ascii, blank]",
                    "[别名: round -> rounded, plain -> ascii]",
                ],
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
            push_option(
                lines,
                emoji,
                "⏳",
                "--no-spinner",
                "禁用交互式加载动画。",
                &[],
            );
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
                "🎨",
                "--no-color",
                "禁用 CLI 输出和 TUI 外框的 ANSI 颜色。",
                &[],
            );
            push_option(
                lines,
                emoji,
                "🔠",
                "--no-bold",
                "禁用 CLI 输出和 TUI 外框的粗体样式。",
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
        Lang::EnUs => {
            push_option(
                lines,
                emoji,
                "🗂️",
                "--tab <TAB>",
                "Initial TUI tab.",
                &[
                    "[default: overview]",
                    "[possible values: overview, physical-disk, logical-disk, memory, cpu, motherboard, warnings]",
                    "[aliases: disk -> physical-disk, physical -> physical-disk, logical -> logical-disk]",
                ],
            );
            push_option(
                lines,
                emoji,
                "📈",
                "--chart-mode <CHART_MODE>",
                "Initial TUI chart mode; z/c keeps cycling in the fixed order.",
                &[
                    "[default: gauge]",
                    "[cycle: gauge, bar, sparkline, line, scatter]",
                ],
            );
            push_option(
                lines,
                emoji,
                "▣",
                "--border <BORDER>",
                "TUI panel border style. Alias: --tui-border.",
                &[
                    "[default: rounded]",
                    "[possible values: rounded, plain, double, thick]",
                    "[aliases: round -> rounded, square -> plain]",
                ],
            );
            push_option(
                lines,
                emoji,
                "⏱️",
                "-t, --interval <INTERVAL>",
                "TUI refresh interval in milliseconds.",
                &["[default: 1000]", "[minimum: 250]"],
            );
        }
        Lang::ZhCn => {
            push_option(
                lines,
                emoji,
                "🗂️",
                "--tab <TAB>",
                "TUI 初始标签页。",
                &[
                    "[默认: overview]",
                    "[可选值: overview, physical-disk, logical-disk, memory, cpu, motherboard, warnings]",
                    "[别名: disk -> physical-disk, physical -> physical-disk, logical -> logical-disk]",
                ],
            );
            push_option(
                lines,
                emoji,
                "📈",
                "--chart-mode <CHART_MODE>",
                "TUI 初始图表模式；z/c 仍按固定顺序循环。",
                &[
                    "[默认: gauge]",
                    "[循环: gauge, bar, sparkline, line, scatter]",
                ],
            );
            push_option(
                lines,
                emoji,
                "▣",
                "--border <BORDER>",
                "TUI 面板边框样式。别名：--tui-border。",
                &[
                    "[默认: rounded]",
                    "[可选值: rounded, plain, double, thick]",
                    "[别名: round -> rounded, square -> plain]",
                ],
            );
            push_option(
                lines,
                emoji,
                "⏱️",
                "-t, --interval <INTERVAL>",
                "TUI 刷新间隔，单位毫秒。",
                &["[默认: 1000]", "[最小值: 250]"],
            );
        }
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
