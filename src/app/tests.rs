use clap::Parser;

use super::cli::Cli;
use super::command::Command;
use super::options::{RenderFormat, TableStyle, TuiBorder};
use super::validate_cli;

fn parse(args: &[&str]) -> Cli {
    Cli::try_parse_from(args.iter().copied()).expect("CLI arguments should parse")
}

#[test]
fn defaults_to_table_with_rounded_style() {
    let cli = parse(&["hdrt"]);

    assert_eq!(cli.format, RenderFormat::Table);
    assert_eq!(cli.style, None);
    assert_eq!(cli.table_style(), TableStyle::Rounded);
}

#[test]
fn parses_all_render_formats() {
    for (value, expected) in [
        ("table", RenderFormat::Table),
        ("json", RenderFormat::Json),
        ("markdown", RenderFormat::Markdown),
    ] {
        assert_eq!(parse(&["hdrt", "--format", value]).format, expected);
    }
}

#[test]
fn parses_all_table_styles() {
    for (value, expected) in [
        ("rounded", TableStyle::Rounded),
        ("modern", TableStyle::Modern),
        ("sharp", TableStyle::Sharp),
        ("psql", TableStyle::Psql),
        ("ascii", TableStyle::Ascii),
        ("blank", TableStyle::Blank),
    ] {
        assert_eq!(parse(&["hdrt", "--style", value]).style, Some(expected));
    }
}

#[test]
fn parses_table_style_flag_and_value_aliases() {
    assert_eq!(
        parse(&["hdrt", "--style", "round"]).style,
        Some(TableStyle::Rounded)
    );
    assert_eq!(
        parse(&["hdrt", "--style", "plain"]).style,
        Some(TableStyle::Ascii)
    );
    assert_eq!(
        parse(&["hdrt", "--table-style", "modern"]).style,
        Some(TableStyle::Modern)
    );
}

#[test]
fn rejects_style_with_non_table_formats() {
    for format in ["json", "markdown"] {
        let cli = parse(&["hdrt", "--format", format, "--style", "rounded"]);
        assert_eq!(
            validate_cli(&cli).unwrap_err().to_string(),
            "--style/--table-style only applies to --format table"
        );
    }
}

#[test]
fn rejects_removed_compact_and_wide_formats() {
    assert!(Cli::try_parse_from(["hdrt", "--format", "compact"]).is_err());
    assert!(Cli::try_parse_from(["hdrt", "--format", "wide"]).is_err());
}

#[test]
fn parses_all_tui_border_styles() {
    for (value, expected) in [
        ("rounded", TuiBorder::Rounded),
        ("plain", TuiBorder::Plain),
        ("double", TuiBorder::Double),
        ("thick", TuiBorder::Thick),
    ] {
        let cli = parse(&["hdrt", "tui", "--border", value]);
        let Some(Command::Tui { border, .. }) = cli.command else {
            panic!("expected tui command");
        };
        assert_eq!(border, expected);
    }
}

#[test]
fn parses_tui_border_flag_and_value_aliases() {
    let rounded = parse(&["hdrt", "tui", "--tui-border", "round"]);
    let plain = parse(&["hdrt", "tui", "--border", "square"]);

    assert!(matches!(
        rounded.command,
        Some(Command::Tui {
            border: TuiBorder::Rounded,
            ..
        })
    ));
    assert!(matches!(
        plain.command,
        Some(Command::Tui {
            border: TuiBorder::Plain,
            ..
        })
    ));
}

#[test]
fn parses_common_subcommand_aliases() {
    assert!(matches!(
        parse(&["hdrt", "pd"]).command,
        Some(Command::PhysicalDisk)
    ));
    assert!(matches!(
        parse(&["hdrt", "ld"]).command,
        Some(Command::LogicalDisk)
    ));
    assert!(matches!(
        parse(&["hdrt", "mem"]).command,
        Some(Command::Memory)
    ));
    assert!(matches!(
        parse(&["hdrt", "mb"]).command,
        Some(Command::Motherboard)
    ));
}

#[test]
fn parses_color_bold_and_debug_flags() {
    let cli = parse(&["hdrt", "--no-color", "--no-bold", "--debug"]);

    assert!(cli.no_color);
    assert!(cli.no_bold);
    assert!(cli.debug);
}
