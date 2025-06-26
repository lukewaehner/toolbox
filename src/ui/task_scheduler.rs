use crate::models::app_state::{AppState, StatusMessageType};
use crate::ui::common::{get_priority_color, get_status_color, get_text_color};
use chrono::Utc;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table, TableState},
    Frame,
};

pub fn draw_task_scheduler_menu<B: Backend>(f: &mut Frame<B>) {
    let text_color = get_text_color();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(f.size());

    let text = vec![
        Spans::from(Span::raw("a. Add Task")),
        Spans::from(Span::raw("v. View Tasks")),
        Spans::from(Span::raw("e. Email Configuration")),
        Spans::from(Span::raw("Esc. Back to Main Menu")),
        Spans::from(Span::raw("Press 'q' to quit")),
    ];
    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .title("Task Scheduler")
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(text_color));
    f.render_widget(paragraph, chunks[0]);
}

pub fn draw_add_task<B: Backend>(f: &mut Frame<B>, app_state: &AppState) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(10),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
            ]
            .as_ref(),
        )
        .split(f.size());

    let highlight_style = Style::default()
        .fg(Color::Yellow)
        .bg(Color::DarkGray)
        .add_modifier(Modifier::BOLD);
    let normal_style = Style::default().fg(Color::White);

    let priority_display = format!("{:?}", app_state.task_priority);

    let fields = [
        ("Title: ", &app_state.task_title, app_state.input_field == 0),
        (
            "Description: ",
            &app_state.task_description,
            app_state.input_field == 1,
        ),
        (
            "Due Date (YYYY-MM-DD): ",
            &app_state.task_due_date,
            app_state.input_field == 2,
        ),
        (
            "Tags (comma-separated): ",
            &app_state.task_tags,
            app_state.input_field == 3,
        ),
    ];

    for (i, (label, value, is_selected)) in fields.iter().enumerate() {
        let text = Spans::from(vec![
            Span::raw(*label),
            Span::styled(
                value.clone(),
                if *is_selected {
                    highlight_style
                } else {
                    normal_style
                },
            ),
        ]);

        // Highlight the entire block if this field is selected
        let block_style = if *is_selected {
            Style::default().fg(Color::Yellow).bg(Color::DarkGray)
        } else {
            Style::default()
        };

        let paragraph =
            Paragraph::new(text).block(Block::default().borders(Borders::ALL).style(block_style));
        f.render_widget(paragraph, layout[i + 1]);
    }

    // Priority field for the add task screen
    let priority_text = Spans::from(vec![
        Span::raw("Priority: "),
        Span::styled(
            priority_display,
            Style::default().fg(get_priority_color(&app_state.task_priority)),
        ),
        Span::raw(" (Use ↑↓ to change)"),
    ]);
    let priority_paragraph =
        Paragraph::new(priority_text).block(Block::default().borders(Borders::ALL));
    f.render_widget(priority_paragraph, layout[5]);

    // Instructions
    let instructions = Paragraph::new("Press 'Enter' to Save, 'Esc' to Cancel")
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(instructions, layout[6]);

    // Display status message if present
    if let Some(ref msg) = app_state.status_message {
        let status_block = Paragraph::new(msg.message.clone())
            .style(Style::default().fg(match msg.message_type {
                StatusMessageType::Info => Color::Blue,
                StatusMessageType::Success => Color::Green,
                StatusMessageType::Warning => Color::Yellow,
                StatusMessageType::Error => Color::Red,
            }))
            .block(Block::default().borders(Borders::ALL).title("Status"));

        // Create a centered popup for the status message
        let area = f.size();
        let status_width = 60.min(area.width.saturating_sub(4));
        let status_height = 5.min(area.height.saturating_sub(4));

        let status_area = Rect::new(
            ((area.width - status_width) / 2).max(0),
            ((area.height - status_height) / 2).max(0),
            status_width,
            status_height,
        );

        f.render_widget(Clear, status_area); // Clear the area
        f.render_widget(status_block, status_area);
    }
}

pub fn draw_view_tasks<B: Backend>(f: &mut Frame<B>, app_state: &AppState) {
    let text_color = get_text_color();

    // Create layout with a status bar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(10),   // Task table
            Constraint::Length(3), // Task details
            Constraint::Length(1), // Status message
            Constraint::Length(3), // Controls
        ])
        .split(f.size());

    // Title
    let title = Paragraph::new(vec![
        Spans::from(vec![Span::styled(
            "TASK MANAGER",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Spans::from(vec![Span::raw(
            "Press ↑↓ to navigate, 'r' to add reminder, 'c' to complete, 'd' to delete, 'Esc' to go back",
        )]),
    ])
    .block(Block::default().borders(Borders::BOTTOM));

    f.render_widget(title, chunks[0]);

    // Task list
    if let Some(ref task_scheduler) = app_state.task_scheduler {
        if let Ok(scheduler) = task_scheduler.lock() {
            let all_tasks = scheduler.get_all_tasks();

            if all_tasks.is_empty() {
                let no_tasks = Paragraph::new("No tasks found. Press 'a' to add a new task.")
                    .style(Style::default().fg(text_color))
                    .block(Block::default().borders(Borders::ALL).title("Tasks"));
                f.render_widget(no_tasks, chunks[1]);
            } else {
                // Sort tasks by due date
                let mut sorted_tasks = all_tasks.clone();
                sorted_tasks.sort_by(|a, b| a.due_date.cmp(&b.due_date));

                // Table headers
                let header_cells = vec![
                    Cell::from("ID").style(Style::default().fg(Color::Yellow)),
                    Cell::from("Title").style(Style::default().fg(Color::Yellow)),
                    Cell::from("Due Date").style(Style::default().fg(Color::Yellow)),
                    Cell::from("Priority").style(Style::default().fg(Color::Yellow)),
                    Cell::from("Status").style(Style::default().fg(Color::Yellow)),
                    Cell::from("Reminders").style(Style::default().fg(Color::Yellow)),
                ];

                let header = Row::new(header_cells)
                    .style(Style::default().add_modifier(Modifier::BOLD))
                    .height(1);

                // Task rows
                let rows = sorted_tasks.iter().map(|task| {
                    let id = task.id.to_string();
                    let title = task.title.clone();
                    let due_date = crate::task_scheduler::format_timestamp(task.due_date);
                    let priority = format!("{:?}", task.priority);
                    let status = format!("{:?}", task.status);
                    let reminders = task.reminders.len().to_string();

                    // Calculate color based on due date
                    let now = Utc::now().timestamp();
                    let row_color = if task.status == crate::task_scheduler::TaskStatus::Completed {
                        Color::DarkGray
                    } else if task.due_date < now {
                        Color::Red
                    } else if task.due_date - now < 86400 {
                        // Within 24 hours
                        Color::Yellow
                    } else {
                        text_color
                    };

                    Row::new(vec![
                        Cell::from(id),
                        Cell::from(title),
                        Cell::from(due_date),
                        Cell::from(priority)
                            .style(Style::default().fg(get_priority_color(&task.priority))),
                        Cell::from(status)
                            .style(Style::default().fg(get_status_color(&task.status))),
                        Cell::from(reminders),
                    ])
                    .style(Style::default().fg(row_color))
                });

                // Create a stateful table
                let mut state = TableState::default();

                // Find the selected task index
                let selected_index = if let Some(selected_id) = app_state.selected_task_id {
                    sorted_tasks.iter().position(|task| task.id == selected_id)
                } else {
                    None
                };

                // Set the selected index
                if let Some(idx) = selected_index {
                    state.select(Some(idx));
                } else if !sorted_tasks.is_empty() {
                    state.select(Some(0));
                }

                let table = Table::new(rows)
                    .header(header)
                    .block(Block::default().title("Tasks").borders(Borders::ALL))
                    .widths(&[
                        Constraint::Length(5),
                        Constraint::Percentage(35),
                        Constraint::Length(19),
                        Constraint::Length(10),
                        Constraint::Length(12),
                        Constraint::Length(10),
                    ])
                    .column_spacing(1)
                    .highlight_style(
                        Style::default()
                            .bg(Color::DarkGray)
                            .add_modifier(Modifier::BOLD),
                    );

                // Render stateful table
                f.render_stateful_widget(table, chunks[1], &mut state);

                // Show task details if a task is selected
                if let Some(selected_id) = app_state.selected_task_id {
                    if let Some(task) = scheduler.get_task(selected_id) {
                        let details = vec![
                            Spans::from(vec![
                                Span::styled(
                                    "Title: ",
                                    Style::default().add_modifier(Modifier::BOLD),
                                ),
                                Span::raw(&task.title),
                            ]),
                            Spans::from(vec![
                                Span::styled(
                                    "Description: ",
                                    Style::default().add_modifier(Modifier::BOLD),
                                ),
                                Span::raw(&task.description),
                            ]),
                            Spans::from(vec![
                                Span::styled(
                                    "Tags: ",
                                    Style::default().add_modifier(Modifier::BOLD),
                                ),
                                Span::raw(task.tags.join(", ")),
                            ]),
                        ];

                        let details_block = Paragraph::new(details)
                            .block(Block::default().title("Task Details").borders(Borders::ALL));
                        f.render_widget(details_block, chunks[2]);
                    }
                }
            }
        }
    } else {
        // If no scheduler is available
        let no_data = Paragraph::new("Task scheduler not initialized")
            .style(Style::default().fg(text_color))
            .block(Block::default().borders(Borders::ALL));

        f.render_widget(no_data, chunks[1]);
    }

    // Status message
    if let Some(ref status) = app_state.status_message {
        let message_color = match status.message_type {
            StatusMessageType::Info => Color::Blue,
            StatusMessageType::Success => Color::Green,
            StatusMessageType::Warning => Color::Yellow,
            StatusMessageType::Error => Color::Red,
        };

        let status_text = Spans::from(vec![
            Span::styled("◆ ", Style::default().fg(message_color)),
            Span::styled(&status.message, Style::default().fg(message_color)),
        ]);

        let status_bar = Paragraph::new(status_text);
        f.render_widget(status_bar, chunks[3]);
    }

    // Controls
    let controls = Paragraph::new(vec![Spans::from(vec![Span::raw(
        "Actions: [r]Add Reminder [c]Complete [d]Delete | [↑↓]Navigate",
    )])])
    .block(Block::default().borders(Borders::TOP));

    f.render_widget(controls, chunks[4]);
}

pub fn draw_email_config<B: Backend>(f: &mut Frame<B>, app_state: &AppState) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(10),
                Constraint::Percentage(13), // Email address
                Constraint::Percentage(13), // SMTP server
                Constraint::Percentage(13), // SMTP port
                Constraint::Percentage(13), // Username
                Constraint::Percentage(13), // Password
                Constraint::Percentage(13), // Test button
                Constraint::Percentage(12), // Instructions
            ]
            .as_ref(),
        )
        .split(f.size());

    let highlight_style = Style::default()
        .fg(Color::Yellow)
        .bg(Color::DarkGray) // More visible background
        .add_modifier(Modifier::BOLD);
    let normal_style = Style::default().fg(Color::White);

    let fields = [
        (
            "Email Address: ",
            &app_state.email_address,
            app_state.email_config_field == 0,
        ),
        (
            "SMTP Server: ",
            &app_state.email_smtp_server,
            app_state.email_config_field == 1,
        ),
        (
            "SMTP Port: ",
            &app_state.email_smtp_port,
            app_state.email_config_field == 2,
        ),
        (
            "Username: ",
            &app_state.email_username,
            app_state.email_config_field == 3,
        ),
        (
            "Password: ",
            &app_state.email_password,
            app_state.email_config_field == 4,
        ),
    ];

    // Title
    let title = Paragraph::new("Email Configuration")
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().add_modifier(Modifier::BOLD));
    f.render_widget(title, layout[0]);

    for (i, (label, value, is_selected)) in fields.iter().enumerate() {
        let display_value = if i == 4 && !value.is_empty() {
            "*".repeat(value.len()) // Mask password
        } else {
            value.to_string()
        };

        let text = Spans::from(vec![
            Span::raw(*label),
            Span::styled(
                display_value,
                if *is_selected {
                    highlight_style
                } else {
                    normal_style
                },
            ),
        ]);

        // Highlight the entire block if this field is selected
        let block_style = if *is_selected {
            Style::default().fg(Color::Yellow).bg(Color::DarkGray)
        } else {
            Style::default()
        };

        let paragraph =
            Paragraph::new(text).block(Block::default().borders(Borders::ALL).style(block_style));

        f.render_widget(paragraph, layout[i + 1]);
    }

    // Add a Test Config button
    let is_test_selected = app_state.email_config_field == 5;
    let test_button_style = if is_test_selected {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Yellow)
    };

    let test_button = Paragraph::new("[ Test Configuration ]")
        .style(test_button_style)
        .alignment(tui::layout::Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

    f.render_widget(test_button, layout[5]);

    // Instructions
    let instructions = Paragraph::new(vec![
        Spans::from("Tab to navigate fields, Enter to Save/Test,"),
        Spans::from("Esc to return to menu"),
    ])
    .block(Block::default().borders(Borders::ALL));

    f.render_widget(instructions, layout[6]);

    // Show status message if present
    if let Some(ref msg) = app_state.status_message {
        let status_block = Paragraph::new(msg.message.clone())
            .style(Style::default().fg(match msg.message_type {
                StatusMessageType::Info => Color::Blue,
                StatusMessageType::Success => Color::Green,
                StatusMessageType::Warning => Color::Yellow,
                StatusMessageType::Error => Color::Red,
            }))
            .block(Block::default().borders(Borders::ALL).title("Status"));

        // Create a centered popup for the status message
        let area = f.size();
        let status_width = 60.min(area.width.saturating_sub(4));
        let status_height = 5.min(area.height.saturating_sub(4));

        let status_area = Rect::new(
            ((area.width - status_width) / 2).max(0),
            ((area.height - status_height) / 2).max(0),
            status_width,
            status_height,
        );

        f.render_widget(Clear, status_area); // Clear the area
        f.render_widget(status_block, status_area);
    }
}

pub fn draw_add_reminder<B: Backend>(f: &mut Frame<B>, app_state: &AppState) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(15),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
                Constraint::Percentage(25),
            ]
            .as_ref(),
        )
        .split(f.size());

    let highlight_style = Style::default().fg(Color::Yellow).bg(Color::Blue);
    let normal_style = Style::default().fg(Color::White);

    // Title with selected task info
    let mut title_text = "Add Reminder".to_string();
    if let Some(task_id) = app_state.selected_task_id {
        if let Some(ref task_scheduler) = app_state.task_scheduler {
            if let Ok(scheduler) = task_scheduler.lock() {
                if let Some(task) = scheduler.get_task(task_id) {
                    title_text = format!("Add Reminder for: {}", task.title);
                }
            }
        }
    }

    let title = Paragraph::new(title_text)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().add_modifier(Modifier::BOLD));
    f.render_widget(title, layout[0]);

    let reminder_type_display = format!("{:?}", app_state.reminder_type);

    let fields = [
        (
            "Date (YYYY-MM-DD): ",
            &app_state.reminder_date,
            app_state.input_field == 0,
        ),
        (
            "Time (HH:MM): ",
            &app_state.reminder_time,
            app_state.input_field == 1,
        ),
    ];

    for (i, (label, value, is_selected)) in fields.iter().enumerate() {
        let text = Spans::from(vec![
            Span::raw(*label),
            Span::styled(
                value.clone(),
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

    // Reminder type selection
    let reminder_type_text = Spans::from(vec![
        Span::raw("Reminder Type: "),
        Span::styled(reminder_type_display, Style::default().fg(Color::Cyan)),
        Span::raw(" (Use ↑↓ to change)"),
    ]);
    let reminder_type_paragraph =
        Paragraph::new(reminder_type_text).block(Block::default().borders(Borders::ALL));
    f.render_widget(reminder_type_paragraph, layout[3]);

    // Instructions
    let instructions = Paragraph::new("Press 'Enter' to Save, 'Esc' to Cancel")
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(instructions, layout[4]);
}

pub fn draw_sms_config<B: Backend>(f: &mut Frame<B>, app_state: &AppState) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(15), // Title
                Constraint::Percentage(20), // Phone number
                Constraint::Percentage(20), // Carrier
                Constraint::Percentage(15), // Enabled toggle
                Constraint::Percentage(15), // Instructions
                Constraint::Percentage(15), // Status message area
            ]
            .as_ref(),
        )
        .split(f.size());

    let highlight_style = Style::default()
        .fg(Color::Yellow)
        .bg(Color::DarkGray)
        .add_modifier(Modifier::BOLD);
    let normal_style = Style::default().fg(Color::White);

    // Title
    let title = Paragraph::new("SMS Configuration")
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().add_modifier(Modifier::BOLD));
    f.render_widget(title, layout[0]);

    // Phone number field
    let phone_text = Spans::from(vec![
        Span::raw("Phone Number: "),
        Span::styled(
            app_state.sms_phone_number.clone(),
            if app_state.sms_config_field == 0 {
                highlight_style
            } else {
                normal_style
            },
        ),
    ]);
    let phone_block_style = if app_state.sms_config_field == 0 {
        Style::default().fg(Color::Yellow).bg(Color::DarkGray)
    } else {
        Style::default()
    };
    let phone_paragraph = Paragraph::new(phone_text)
        .block(Block::default().borders(Borders::ALL).style(phone_block_style));
    f.render_widget(phone_paragraph, layout[1]);

    // Carrier field
    let carrier_text = Spans::from(vec![
        Span::raw("Carrier: "),
        Span::styled(
            app_state.sms_carrier.clone(),
            if app_state.sms_config_field == 1 {
                highlight_style
            } else {
                normal_style
            },
        ),
        Span::raw(" (Use ↑↓ to change)"),
    ]);
    let carrier_block_style = if app_state.sms_config_field == 1 {
        Style::default().fg(Color::Yellow).bg(Color::DarkGray)
    } else {
        Style::default()
    };
    let carrier_paragraph = Paragraph::new(carrier_text)
        .block(Block::default().borders(Borders::ALL).style(carrier_block_style));
    f.render_widget(carrier_paragraph, layout[2]);

    // Enabled toggle
    let enabled_text = Spans::from(vec![
        Span::raw("SMS Enabled: "),
        Span::styled(
            if app_state.sms_enabled { "✓ Yes" } else { "✗ No" }.to_string(),
            if app_state.sms_enabled {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Red)
            },
        ),
        Span::raw(" (Press Enter/Space to toggle)"),
    ]);
    let enabled_block_style = if app_state.sms_config_field == 2 {
        Style::default().fg(Color::Yellow).bg(Color::DarkGray)
    } else {
        Style::default()
    };
    let enabled_paragraph = Paragraph::new(enabled_text)
        .block(Block::default().borders(Borders::ALL).style(enabled_block_style));
    f.render_widget(enabled_paragraph, layout[3]);

    // Instructions
    let instructions = Paragraph::new(vec![
        Spans::from("Tab to navigate fields, Enter to Save,"),
        Spans::from("Esc to return to menu"),
        Spans::from("Supported carriers: AT&T, Verizon, T-Mobile, Sprint, Boost, Cricket, MetroPCS"),
    ])
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(instructions, layout[4]);

    // Show status message if present
    if let Some(ref msg) = app_state.status_message {
        let status_block = Paragraph::new(msg.message.clone())
            .style(Style::default().fg(match msg.message_type {
                StatusMessageType::Info => Color::Blue,
                StatusMessageType::Success => Color::Green,
                StatusMessageType::Warning => Color::Yellow,
                StatusMessageType::Error => Color::Red,
            }))
            .block(Block::default().borders(Borders::ALL).title("Status"));

        f.render_widget(status_block, layout[5]);
    }
}
