use crate::models::app_state::{AppState, ConfirmationDialogue, ProcessSortType, SystemViewMode};
use crate::ui::common::{get_text_color, get_usage_color};
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Cell, Clear, Gauge, Paragraph, Row, Table, TableState},
    Frame,
};

pub fn draw_system_utilities_menu<B: Backend>(f: &mut Frame<B>) {
    let text_color = get_text_color();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(f.size());

    let text = vec![
        Spans::from(Span::raw("r. Resource Monitor")),
        Spans::from(Span::raw("p. Process Manager")),
        Spans::from(Span::raw("d. Disk Space Analyzer")),
        Spans::from(Span::raw("Esc. Back to Main Menu")),
        Spans::from(Span::raw("Press 'q' to quit")),
    ];

    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .title("System Utilities")
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(text_color));

    f.render_widget(paragraph, chunks[0]);
}

pub fn draw_resource_monitor<B: Backend>(f: &mut Frame<B>, app_state: &AppState) {
    let text_color = get_text_color();

    // Create a layout for different sections
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(9), // CPU
            Constraint::Length(9), // Memory
            Constraint::Min(10),   // Processes
        ])
        .split(f.size());

    // Title
    let title = Paragraph::new(vec![
        Spans::from(vec![
            Span::styled("SYSTEM RESOURCE MONITOR", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Spans::from(vec![
            Span::raw("Press 'c' for CPU details, 'm' for Memory details, 'p' for Process list, 'Esc' to go back"),
        ]),
    ])
    .block(Block::default().borders(Borders::BOTTOM));

    f.render_widget(title, chunks[0]);

    // If we have system data, show it
    if let Some(ref snapshot) = app_state.system_snapshot {
        // CPU Usage section
        let cpu_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(chunks[1]);

        // CPU usage gauge
        let cpu_percent = snapshot.cpu_usage as u16;
        let cpu_gauge = Gauge::default()
            .block(Block::default().title("CPU Usage").borders(Borders::ALL))
            .gauge_style(
                Style::default()
                    .fg(get_usage_color(cpu_percent as f32))
                    .bg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            )
            .percent(cpu_percent);

        f.render_widget(cpu_gauge, cpu_layout[0]);

        // CPU info
        let cpu_info = Paragraph::new(vec![
            Spans::from(vec![Span::raw(format!(
                "Cores: {}",
                snapshot.cpu_cores_count
            ))]),
            Spans::from(vec![Span::raw(format!("Model: {}", snapshot.cpu_name))]),
        ])
        .block(Block::default().title("CPU Info").borders(Borders::ALL));

        f.render_widget(cpu_info, cpu_layout[1]);

        // Memory section
        let mem_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(chunks[2]);

        // Memory usage gauge
        let mem_percent = snapshot.memory_usage_percent as u16;
        let mem_gauge = Gauge::default()
            .block(Block::default().title("Memory Usage").borders(Borders::ALL))
            .gauge_style(
                Style::default()
                    .fg(get_usage_color(mem_percent as f32))
                    .bg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            )
            .percent(mem_percent);

        f.render_widget(mem_gauge, mem_layout[0]);

        // Memory info
        let mem_info = Paragraph::new(vec![
            Spans::from(vec![Span::raw(format!(
                "Used: {} MB",
                snapshot.memory_used / 1024 / 1024
            ))]),
            Spans::from(vec![Span::raw(format!(
                "Total: {} MB",
                snapshot.memory_total / 1024 / 1024
            ))]),
            Spans::from(vec![Span::raw(format!(
                "Swap Used: {} MB",
                snapshot.swap_used / 1024 / 1024
            ))]),
            Spans::from(vec![Span::raw(format!(
                "Swap Total: {} MB",
                snapshot.swap_total / 1024 / 1024
            ))]),
        ])
        .block(Block::default().title("Memory Info").borders(Borders::ALL));

        f.render_widget(mem_info, mem_layout[1]);

        // Process list
        let process_chunk = chunks[3];
        draw_process_list(f, snapshot, process_chunk);
    } else {
        // If no snapshot is available
        let no_data = Paragraph::new("Loading system data...")
            .style(Style::default().fg(text_color))
            .block(Block::default().borders(Borders::ALL));

        f.render_widget(no_data, chunks[1]);
    }
}

pub fn draw_process_list<B: Backend>(
    f: &mut Frame<B>,
    snapshot: &crate::system_utilities::SystemSnapshot,
    area: Rect,
) {
    // Table headers
    let header_cells = ["PID", "Name", "CPU %", "Memory", "Mem %", "Runtime"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));

    let header = Row::new(header_cells)
        .style(Style::default().add_modifier(Modifier::BOLD))
        .height(1);

    // Process rows
    let rows = snapshot.top_processes.iter().map(|process| {
        let pid = process.pid.to_string();
        let name = process.name.clone();
        let cpu = format!("{:.1}%", process.cpu_usage);
        let mem = format!("{} MB", process.memory_usage / 1024 / 1024);
        let mem_percent = format!("{:.1}%", process.memory_usage_percent);

        // Format runtime
        let hours = process.run_time / 3600;
        let minutes = (process.run_time % 3600) / 60;
        let seconds = process.run_time % 60;
        let runtime = format!("{:02}:{:02}:{:02}", hours, minutes, seconds);

        Row::new(vec![
            Cell::from(pid),
            Cell::from(name),
            Cell::from(cpu),
            Cell::from(mem),
            Cell::from(mem_percent),
            Cell::from(runtime),
        ])
    });

    let table = Table::new(rows)
        .header(header)
        .block(
            Block::default()
                .title("Top Processes")
                .borders(Borders::ALL),
        )
        .widths(&[
            Constraint::Length(8),
            Constraint::Percentage(30),
            Constraint::Length(8),
            Constraint::Length(10),
            Constraint::Length(8),
            Constraint::Length(10),
        ])
        .column_spacing(1);

    f.render_widget(table, area);
}

pub fn draw_process_list_detailed<B: Backend>(f: &mut Frame<B>, app_state: &AppState) {
    let text_color = get_text_color();

    // Create layout with a status bar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(10),   // Process table
            Constraint::Length(1), // Status message
            Constraint::Length(3), // Controls
        ])
        .split(f.size());

    // Title
    let title = Paragraph::new(vec![
        Spans::from(vec![Span::styled(
            "PROCESS MANAGER",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Spans::from(vec![Span::raw(
            "Press ↑↓ to navigate, 'k' to kill selected process, 'r' to refresh, 'Esc' to go back",
        )]),
    ])
    .block(Block::default().borders(Borders::BOTTOM));

    f.render_widget(title, chunks[0]);

    // Process list
    if let Some(ref snapshot) = app_state.system_snapshot {
        // Table headers with highlighting for current sort
        let sort_highlight = Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED);
        let normal_header = Style::default().fg(Color::Yellow);

        let pid_style = if app_state.process_sort_type == ProcessSortType::Pid {
            sort_highlight
        } else {
            normal_header
        };
        let name_style = if app_state.process_sort_type == ProcessSortType::Name {
            sort_highlight
        } else {
            normal_header
        };
        let cpu_style = if app_state.process_sort_type == ProcessSortType::CpuUsage {
            sort_highlight
        } else {
            normal_header
        };
        let mem_style = if app_state.process_sort_type == ProcessSortType::MemoryUsage {
            sort_highlight
        } else {
            normal_header
        };
        let runtime_style = if app_state.process_sort_type == ProcessSortType::Runtime {
            sort_highlight
        } else {
            normal_header
        };

        let header_cells = vec![
            Cell::from("PID").style(pid_style),
            Cell::from("Name").style(name_style),
            Cell::from("CPU %").style(cpu_style),
            Cell::from("Memory").style(mem_style),
            Cell::from("Mem %").style(mem_style),
            Cell::from("Disk I/O").style(normal_header),
            Cell::from("Start Time").style(normal_header),
            Cell::from("Runtime").style(runtime_style),
        ];

        let header = Row::new(header_cells)
            .style(Style::default().add_modifier(Modifier::BOLD))
            .height(1);

        // Get a sorted copy of processes
        let mut sorted_processes = snapshot.top_processes.clone();
        match app_state.process_sort_type {
            ProcessSortType::Pid => {
                sorted_processes.sort_by_key(|p| p.pid);
            }
            ProcessSortType::Name => {
                sorted_processes.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
            }
            ProcessSortType::CpuUsage => {
                sorted_processes.sort_by(|a, b| {
                    b.cpu_usage
                        .partial_cmp(&a.cpu_usage)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            ProcessSortType::MemoryUsage => {
                sorted_processes.sort_by(|a, b| b.memory_usage.cmp(&a.memory_usage));
            }
            ProcessSortType::Runtime => {
                sorted_processes.sort_by(|a, b| b.run_time.cmp(&a.run_time));
            }
        }

        // Process rows
        let rows = sorted_processes.iter().map(|process| {
            let pid = process.pid.to_string();
            let name = process.name.clone();
            let cpu = format!("{:.1}%", process.cpu_usage);
            let mem = format!("{} MB", process.memory_usage / 1024 / 1024);
            let mem_percent = format!("{:.1}%", process.memory_usage_percent);
            let disk_io = format!("{} KB", process.disk_usage / 1024);

            // Format start time
            let start_datetime = chrono::DateTime::from_timestamp(process.start_time as i64, 0)
                .map(|dt| dt.format("%H:%M:%S").to_string())
                .unwrap_or_else(|| "Unknown".to_string());

            // Format runtime
            let hours = process.run_time / 3600;
            let minutes = (process.run_time % 3600) / 60;
            let seconds = process.run_time % 60;
            let runtime = format!("{:02}:{:02}:{:02}", hours, minutes, seconds);

            Row::new(vec![
                Cell::from(pid),
                Cell::from(name),
                Cell::from(cpu),
                Cell::from(mem),
                Cell::from(mem_percent),
                Cell::from(disk_io),
                Cell::from(start_datetime),
                Cell::from(runtime),
            ])
        });

        // Create a stateful table
        let mut state = TableState::default();

        // Find the currently selected process by PID
        let selected_index = if let Some(selected_pid) = app_state.selected_process_pid {
            sorted_processes.iter().position(|p| p.pid == selected_pid)
        } else {
            None
        };

        // Set the selected index
        if let Some(idx) = selected_index {
            state.select(Some(idx));
        } else if !sorted_processes.is_empty() {
            state.select(Some(0));
        }

        let table = Table::new(rows)
            .header(header)
            .block(Block::default().title("Processes").borders(Borders::ALL))
            .widths(&[
                Constraint::Length(8),
                Constraint::Percentage(25),
                Constraint::Length(8),
                Constraint::Length(10),
                Constraint::Length(8),
                Constraint::Length(10),
                Constraint::Length(10),
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
    } else {
        // If no snapshot is available
        let no_data = Paragraph::new("Loading process data...")
            .style(Style::default().fg(text_color))
            .block(Block::default().borders(Borders::ALL));

        f.render_widget(no_data, chunks[1]);
    }

    // Status message (between table and controls)
    if let Some(ref status) = app_state.status_message {
        let message_color = match status.message_type {
            crate::models::app_state::StatusMessageType::Info => Color::Blue,
            crate::models::app_state::StatusMessageType::Success => Color::Green,
            crate::models::app_state::StatusMessageType::Warning => Color::Yellow,
            crate::models::app_state::StatusMessageType::Error => Color::Red,
        };

        let status_text = Spans::from(vec![
            Span::styled("◆ ", Style::default().fg(message_color)),
            Span::styled(&status.message, Style::default().fg(message_color)),
        ]);

        let status_bar = Paragraph::new(status_text);
        f.render_widget(status_bar, chunks[2]);
    }

    // Controls
    let controls = Paragraph::new(vec![Spans::from(vec![Span::raw(
        "Sort: [p]PID [n]Name [c]CPU [m]Memory [t]Time | [r]Refresh [k]Kill [↑↓]Navigate",
    )])])
    .block(Block::default().borders(Borders::TOP));

    f.render_widget(controls, chunks[3]);

    // Draw the confirmation dialog if active
    if app_state.confirmation_dialogue != ConfirmationDialogue::None {
        draw_confirmation_dialogue(f, app_state);
    }
}

pub fn draw_confirmation_dialogue<B: Backend>(f: &mut Frame<B>, app_state: &AppState) {
    if let ConfirmationDialogue::KillProcess(pid, ref name) = &app_state.confirmation_dialogue {
        // Create a centered box for the dialog
        let area = f.size();
        let dialog_width = 60.min(area.width.saturating_sub(4));
        let dialog_height = 10.min(area.height.saturating_sub(4));

        let dialog_area = Rect::new(
            ((area.width - dialog_width) / 2).max(0),
            ((area.height - dialog_height) / 2).max(0),
            dialog_width,
            dialog_height,
        );

        // Draw dialog box
        let dialog = Block::default()
            .title("Confirm Process Termination")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red));

        f.render_widget(Clear, dialog_area); // Clear the area
        f.render_widget(dialog, dialog_area);

        // Dialog content
        let content_area = Rect::new(
            dialog_area.x + 2,
            dialog_area.y + 2,
            dialog_area.width.saturating_sub(4),
            dialog_area.height.saturating_sub(4),
        );

        let message = vec![
            Spans::from(vec![
                Span::raw("Are you sure you want to terminate process "),
                Span::styled(name.clone(), Style::default().fg(Color::Yellow)),
                Span::raw(" (PID: "),
                Span::styled(pid.to_string(), Style::default().fg(Color::Yellow)),
                Span::raw(")?"),
            ]),
            Spans::from(Span::raw("")),
            Spans::from(Span::styled(
                "This action cannot be undone.",
                Style::default().fg(Color::Red),
            )),
            Spans::from(Span::raw("")),
            Spans::from(vec![
                Span::styled("y", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" - Yes, terminate the process"),
            ]),
            Spans::from(vec![
                Span::styled("n", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" - No, cancel"),
            ]),
        ];

        let content = Paragraph::new(message).alignment(Alignment::Center);

        f.render_widget(content, content_area);
    }
}

pub fn draw_disk_analyzer<B: Backend>(f: &mut Frame<B>, app_state: &AppState) {
    let text_color = get_text_color();

    // Create layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(10),   // Disk info
        ])
        .split(f.size());

    // Title
    let title = Paragraph::new(vec![
        Spans::from(vec![Span::styled(
            "DISK SPACE ANALYZER",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Spans::from(vec![Span::raw("Press 'r' to refresh, 'Esc' to go back")]),
    ])
    .block(Block::default().borders(Borders::BOTTOM));

    f.render_widget(title, chunks[0]);

    // Disk info
    if let Some(ref snapshot) = app_state.system_snapshot {
        if snapshot.disks.is_empty() {
            let no_disks = Paragraph::new("No disks found")
                .style(Style::default().fg(text_color))
                .block(Block::default().borders(Borders::ALL));

            f.render_widget(no_disks, chunks[1]);
        } else {
            // Calculate layout for disks
            let disk_count = snapshot.disks.len();
            let disk_constraints = vec![Constraint::Length(4); disk_count];

            let disk_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(disk_constraints)
                .split(chunks[1]);

            // Render each disk
            for (i, disk) in snapshot.disks.iter().enumerate() {
                let disk_layout = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
                    .split(disk_chunks[i]);

                // Disk usage gauge
                let disk_percent = disk.usage_percent as u16;
                let disk_gauge = Gauge::default()
                    .block(
                        Block::default()
                            .title(format!("{} ({})", disk.name, disk.mount_point))
                            .borders(Borders::ALL),
                    )
                    .gauge_style(
                        Style::default()
                            .fg(get_usage_color(disk_percent as f32))
                            .bg(Color::Black)
                            .add_modifier(Modifier::BOLD),
                    )
                    .percent(disk_percent);

                f.render_widget(disk_gauge, disk_layout[0]);

                // Disk details
                let used_gb = (disk.total_space - disk.available_space) as f64 / 1_073_741_824.0; // Convert to GB
                let total_gb = disk.total_space as f64 / 1_073_741_824.0;

                let disk_details = Paragraph::new(vec![
                    Spans::from(vec![Span::raw(format!("Used: {:.1} GB", used_gb))]),
                    Spans::from(vec![Span::raw(format!(
                        "Free: {:.1} GB",
                        disk.available_space as f64 / 1_073_741_824.0
                    ))]),
                    Spans::from(vec![Span::raw(format!("Total: {:.1} GB", total_gb))]),
                    Spans::from(vec![Span::raw(format!("Type: {}", disk.filesystem_type))]),
                ])
                .block(Block::default().title("Details").borders(Borders::ALL));

                f.render_widget(disk_details, disk_layout[1]);
            }
        }
    } else {
        // If no snapshot is available
        let no_data = Paragraph::new("Loading disk data...")
            .style(Style::default().fg(text_color))
            .block(Block::default().borders(Borders::ALL));

        f.render_widget(no_data, chunks[1]);
    }
}

pub fn initialize_process_selection(app_state: &mut AppState) {
    // Only initialize if we don't have a process selected yet
    if app_state.selected_process_pid.is_none() {
        if let Some(ref snapshot) = app_state.system_snapshot {
            if !snapshot.top_processes.is_empty() {
                // Sort processes based on current sort type
                let mut sorted_processes = snapshot.top_processes.clone();
                match app_state.process_sort_type {
                    ProcessSortType::Pid => {
                        sorted_processes.sort_by_key(|p| p.pid);
                    }
                    ProcessSortType::Name => {
                        sorted_processes
                            .sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
                    }
                    ProcessSortType::CpuUsage => {
                        sorted_processes.sort_by(|a, b| {
                            b.cpu_usage
                                .partial_cmp(&a.cpu_usage)
                                .unwrap_or(std::cmp::Ordering::Equal)
                        });
                    }
                    ProcessSortType::MemoryUsage => {
                        sorted_processes.sort_by(|a, b| b.memory_usage.cmp(&a.memory_usage));
                    }
                    ProcessSortType::Runtime => {
                        sorted_processes.sort_by(|a, b| b.run_time.cmp(&a.run_time));
                    }
                }

                // Select the first process in the sorted list
                if !sorted_processes.is_empty() {
                    app_state.selected_process_pid = Some(sorted_processes[0].pid);
                }
            }
        }
    }
}
