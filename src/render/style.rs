use crate::app::options::TableStyle;

#[derive(Debug, Clone, Copy)]
pub(super) struct TextStyle {
    color: bool,
    bold: bool,
}

impl TextStyle {
    pub(super) fn new(color: bool, bold: bool) -> Self {
        Self { color, bold }
    }

    pub(super) fn title(self, text: impl AsRef<str>) -> String {
        self.apply(text.as_ref(), Some("36"), true)
    }

    pub(super) fn header(self, text: impl AsRef<str>) -> String {
        self.apply(text.as_ref(), Some("36"), true)
    }

    pub(super) fn key(self, text: impl AsRef<str>) -> String {
        self.apply(text.as_ref(), Some("36"), false)
    }

    pub(super) fn warning(self, text: impl AsRef<str>) -> String {
        self.apply(text.as_ref(), Some("33"), true)
    }

    pub(super) fn note(self, text: impl AsRef<str>) -> String {
        self.apply(text.as_ref(), Some("90"), false)
    }

    fn apply(self, text: &str, color_code: Option<&str>, bold: bool) -> String {
        let mut codes = Vec::new();
        if self.bold && bold {
            codes.push("1");
        }
        if self.color {
            if let Some(color_code) = color_code {
                codes.push(color_code);
            }
        }

        if codes.is_empty() {
            text.to_string()
        } else {
            format!("\x1b[{}m{text}\x1b[0m", codes.join(";"))
        }
    }
}

pub(super) fn style_table_header(table: String, table_style: TableStyle, style: TextStyle) -> String {
    let header_index = match table_style {
        TableStyle::Psql | TableStyle::Blank => 0,
        _ => 1,
    };

    table
        .lines()
        .enumerate()
        .map(|(index, line)| {
            if index == header_index {
                style.header(line)
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}
