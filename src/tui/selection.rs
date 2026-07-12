use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;

pub(super) fn disk_marker(selected: bool) -> Span<'static> {
    if selected {
        Span::styled(
            "> ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled("  ", Style::default().fg(Color::DarkGray))
    }
}

pub(super) fn disk_item_style(selected: bool) -> Style {
    if selected {
        Style::default().bg(Color::DarkGray)
    } else {
        Style::default()
    }
}

pub(super) fn append_disk_kv(
    spans: &mut Vec<Span<'static>>,
    key: &str,
    value: String,
    selected: bool,
    value_style: Style,
) {
    spans.push(Span::styled(format!("{key}: "), disk_key_style(selected)));
    spans.push(Span::styled(value, value_style));
}

pub(super) fn append_disk_sep(spans: &mut Vec<Span<'static>>) {
    spans.push(Span::styled(" | ", Style::default().fg(Color::DarkGray)));
}

fn disk_key_style(selected: bool) -> Style {
    let mut style = Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD);
    if selected {
        style = style.add_modifier(Modifier::UNDERLINED);
    }
    style
}

pub(super) fn disk_value_style(selected: bool) -> Style {
    let mut style = Style::default().fg(Color::White);
    if selected {
        style = style.add_modifier(Modifier::BOLD);
    }
    style
}

pub(super) fn disk_heading_value_style(selected: bool) -> Style {
    disk_value_style(selected).add_modifier(Modifier::BOLD)
}

pub(super) fn disk_rate_style(selected: bool, color: Color) -> Style {
    let mut style = Style::default().fg(color);
    if selected {
        style = style.add_modifier(Modifier::BOLD);
    }
    style
}
