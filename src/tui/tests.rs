use ratatui::backend::TestBackend;
use ratatui::Terminal;

use crate::app::options::TuiBorder;

use super::style::TuiStyle;

fn assert_border_corners(border: TuiBorder, expected: [&str; 4]) {
    let backend = TestBackend::new(12, 4);
    let mut terminal = Terminal::new(backend).expect("test terminal should initialize");

    terminal
        .draw(|frame| {
            frame.render_widget(TuiStyle::new(border, false, false).block(), frame.area());
        })
        .expect("test border should render");

    let buffer = terminal.backend().buffer();
    assert_eq!(buffer[(0, 0)].symbol(), expected[0]);
    assert_eq!(buffer[(11, 0)].symbol(), expected[1]);
    assert_eq!(buffer[(0, 3)].symbol(), expected[2]);
    assert_eq!(buffer[(11, 3)].symbol(), expected[3]);
}

#[test]
fn rounded_border_renders_rounded_corners() {
    assert_border_corners(TuiBorder::Rounded, ["╭", "╮", "╰", "╯"]);
}

#[test]
fn plain_border_renders_square_corners() {
    assert_border_corners(TuiBorder::Plain, ["┌", "┐", "└", "┘"]);
}

#[test]
fn double_border_renders_double_corners() {
    assert_border_corners(TuiBorder::Double, ["╔", "╗", "╚", "╝"]);
}

#[test]
fn thick_border_renders_thick_corners() {
    assert_border_corners(TuiBorder::Thick, ["┏", "┓", "┗", "┛"]);
}
