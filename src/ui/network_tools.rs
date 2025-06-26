use crate::models::app_state::AppState;
use crate::models::ping_result::PingResult;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

pub fn draw_network_tools_menu<B: Backend>(f: &mut Frame<B>) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(f.size());

    let text = vec![
        Spans::from(Span::raw("p. Ping")),
        Spans::from(Span::raw("t. Traceroute")),
        Spans::from(Span::raw("s. Speed Test")),
        Spans::from(Span::raw("Esc. Back to Main Menu")),
        Spans::from(Span::raw("Press 'q' to quit")),
    ];
    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .title("Network Tools")
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(Color::Black));
    f.render_widget(paragraph, chunks[0]);
}

pub fn draw_address_input<B: Backend>(f: &mut Frame<B>, app_state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(f.size());

    let tool_name = app_state.selected_tool.as_deref().unwrap_or("Unknown Tool");
    let text = vec![
        Spans::from(Span::raw(format!("Enter address for {}:", tool_name))),
        Spans::from(Span::raw(&app_state.address)),
    ];
    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .title("Address Input")
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(Color::White));
    f.render_widget(paragraph, chunks[0]);
}

pub fn draw_speed_test<B: Backend>(f: &mut Frame<B>, app_state: &AppState) {
    // ... implementation of draw_speed_test function ...
}

pub fn draw_view_results<B: Backend>(f: &mut Frame<B>, app_state: &AppState) {
    if let Some(ref result) = app_state.result {
        // Deserialize the PingResult
        if let Ok(ping_result) = serde_json::from_str::<PingResult>(result) {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(100)].as_ref())
                .split(f.size());

            let mut rows = vec![
                Row::new(vec![
                    Cell::from("Packets Transmitted"),
                    Cell::from(ping_result.packets_transmitted.to_string()),
                ]),
                Row::new(vec![
                    Cell::from("Packets Received"),
                    Cell::from(ping_result.packets_received.to_string()),
                ]),
                Row::new(vec![
                    Cell::from("Packet Loss"),
                    Cell::from(format!("{}%", ping_result.packet_loss)),
                ]),
            ];

            if let Some(time) = ping_result.time {
                rows.push(Row::new(vec![
                    Cell::from("Total Time"),
                    Cell::from(format!("{} ms", time)),
                ]));
            }

            // Add more rows for round trip stats...

            let table = Table::new(rows)
                .header(Row::new(vec!["Metric", "Value"]).style(Style::default().fg(Color::Yellow)))
                .block(Block::default().title("Ping Results").borders(Borders::ALL))
                .widths(&[Constraint::Length(20), Constraint::Length(20)]);

            f.render_widget(table, chunks[0]);
        } else {
            // If deserialization fails, display the raw output
            let paragraph = Paragraph::new(result.clone())
                .block(Block::default().title("Results").borders(Borders::ALL))
                .style(Style::default().fg(Color::White));
            f.render_widget(paragraph, f.size());
        }
    } else {
        let paragraph = Paragraph::new("No results available.")
            .block(Block::default().title("Results").borders(Borders::ALL))
            .style(Style::default().fg(Color::White));
        f.render_widget(paragraph, f.size());
    }
}
