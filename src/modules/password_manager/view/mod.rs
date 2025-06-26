use crate::models::app_state::AppState;
use crate::password_manager::retrieve_password;
use crate::ui::common::get_text_color;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn draw_password_manager_menu<B: Backend>(f: &mut Frame<B>) {
    let text_color = get_text_color();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(f.size());

    let text = vec![
        Spans::from(Span::raw("a. Add Password")),
        Spans::from(Span::raw("v. View Passwords")),
        Spans::from(Span::raw("Esc. Back to Main Menu")),
        Spans::from(Span::raw("Press 'q' to quit")),
    ];
    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .title("Password Manager")
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(text_color));
    f.render_widget(paragraph, chunks[0]);
}

pub fn draw_input_modal<B: Backend>(f: &mut Frame<B>, app_state: &AppState) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(10),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(10),
            ]
            .as_ref(),
        )
        .split(f.size());

    let highlight_style = Style::default().fg(Color::Yellow).bg(Color::Blue);
    let normal_style = Style::default().fg(Color::White);

    let fields = [
        ("Service: ", &app_state.service, app_state.input_field == 0),
        (
            "Username: ",
            &app_state.username,
            app_state.input_field == 1,
        ),
        (
            "Password: ",
            &app_state.password,
            app_state.input_field == 2,
        ),
    ];

    for (i, (label, value, is_selected)) in fields.iter().enumerate() {
        let text = Spans::from(vec![
            Span::raw(*label),
            Span::styled(
                (*value).clone(),
                if *is_selected {
                    highlight_style
                } else {
                    normal_style
                },
            ),
        ]);
        let paragraph = Paragraph::new(text).block(Block::default().borders(Borders::ALL));
        f.render_widget(paragraph, layout[i + 1]);
    }

    // Display error or success message if present
    if let Some(ref msg) = app_state.error_message {
        let status_block = Paragraph::new(msg.clone())
            .style(Style::default().fg(Color::Red))
            .block(Block::default().borders(Borders::ALL).title("Status"));
        f.render_widget(status_block, layout[5]); // Adjust index based on your layout
    }

    let instructions = Paragraph::new("Press 'Enter' to Save, 'Esc' to Cancel")
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(instructions, layout[4]);
}

pub fn draw_password_list<B: Backend>(f: &mut Frame<B>) {
    let text_color = get_text_color();

    match retrieve_password() {
        Ok(passwords) => {
            if passwords.is_empty() {
                let paragraph = Paragraph::new("No passwords found.")
                    .block(
                        Block::default()
                            .title("Stored Passwords")
                            .borders(Borders::ALL),
                    )
                    .style(Style::default().fg(Color::Black));
                f.render_widget(paragraph, f.size());
            } else {
                let text: Vec<Spans> = passwords
                    .iter()
                    .map(|entry| {
                        Spans::from(vec![
                            Span::raw(format!("Service: {}, ", entry.service)),
                            Span::raw(format!("Username: {}, ", entry.username)),
                            Span::raw(format!("Password: {}", entry.password)),
                        ])
                    })
                    .collect();

                let paragraph = Paragraph::new(text)
                    .block(
                        Block::default()
                            .title("Stored Passwords")
                            .borders(Borders::ALL),
                    )
                    .style(Style::default().fg(text_color));
                f.render_widget(paragraph, f.size());
            }
        }
        Err(e) => {
            let paragraph = Paragraph::new(format!("Error retrieving passwords: {:?}", e))
                .block(Block::default().title("Error").borders(Borders::ALL))
                .style(Style::default().fg(Color::Red));
            f.render_widget(paragraph, f.size());
        }
    }
}
