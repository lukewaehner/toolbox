use crate::ui::common::get_text_color;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::Style,
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn draw_main_menu<B: Backend>(f: &mut Frame<B>) {
    let text_color = get_text_color();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(f.size());

    let text = vec![
        Spans::from(Span::raw("1. Password Manager")),
        Spans::from(Span::raw("2. Network Tools")),
        Spans::from(Span::raw("3. System Utilities")),
        Spans::from(Span::raw("4. Task Manager")),
        Spans::from(Span::raw("Press 'q' to quit")),
    ];
    let paragraph = Paragraph::new(text)
        .block(Block::default().title("Main Menu").borders(Borders::ALL))
        .style(Style::default().fg(text_color)); // Use dynamic text color
    f.render_widget(paragraph, chunks[0]);
}
