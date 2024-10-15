mod password_manager;
mod network_tools;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use network_tools::ping;
use password_manager::{save_password, PasswordEntry};
use signal_hook::consts::SIGINT;
use signal_hook::flag;
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use std::process::Command;

// Define the PingResult struct
#[derive(serde::Deserialize, Debug)]
struct PingResult {
    packets_transmitted: u32,
    packets_received: u32,
    packet_loss: f32,
    time: Option<u32>, // Change this to Option<u32> if it can be None
    round_trip_min: f32,
    round_trip_avg: f32,
    round_trip_max: f32,
    round_trip_mdev: f32,
}

#[derive(Debug, Clone, PartialEq)]
enum InputMode {
    Normal,
    Editing,
    Viewing,
    EnterAddress,
    ViewResults,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum MenuItem {
    Main,
    PasswordManager,
    NetworkTools,
}

#[derive(Debug, Clone)]
struct AppState {
    active_menu: MenuItem,
    input_mode: InputMode,
    service: String,
    username: String,
    password: String,
    input_field: usize,
    error_message: Option<String>,
    address: String,
    result: Option<String>,
    selected_tool: Option<String>, // New field to store selected network tool
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            active_menu: MenuItem::Main,
            input_mode: InputMode::Normal,
            service: String::new(),
            username: String::new(),
            password: String::new(),
            input_field: 0,
            error_message: None,
            address: String::new(),
            result: None,
            selected_tool: None,
        }
    }
}

// Struct to handle terminal cleanup
struct TerminalCleanup;

impl Drop for TerminalCleanup {
    fn drop(&mut self) {
        // Ensure terminal is reset
        if let Err(e) = disable_raw_mode() {
            eprintln!("Failed to disable raw mode: {:?}", e);
        }
        if let Err(e) = execute!(
            io::stdout(),
            LeaveAlternateScreen,
            DisableMouseCapture // Ensure mouse capture is disabled
        ) {
            eprintln!("Failed to leave alternate screen or disable mouse capture: {:?}", e);
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    // Instantiate TerminalCleanup to ensure it is used
    let _cleanup = TerminalCleanup;

    // Setup signal handling for Ctrl+C
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    flag::register(SIGINT, r)?;

    let backend = tui::backend::CrosstermBackend::new(stdout);
    let mut terminal = tui::Terminal::new(backend)?;

    let res = run_app(&mut terminal, running);

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, running: Arc<AtomicBool>) -> io::Result<()> {
    let mut app_state = AppState::default();

    while running.load(Ordering::Relaxed) {
        terminal.draw(|f| match app_state.active_menu {
            MenuItem::Main => draw_main_menu(f),
            MenuItem::PasswordManager => match app_state.input_mode {
                InputMode::Normal => draw_password_manager_menu(f),
                InputMode::Editing => draw_input_modal(f, &app_state),
                InputMode::Viewing => draw_password_list(f),
                _ => {}
            },
            MenuItem::NetworkTools => match app_state.input_mode {
                InputMode::Normal => draw_network_tools_menu(f),
                InputMode::EnterAddress => draw_address_input(f, &app_state),
                InputMode::ViewResults => draw_view_results(f, &app_state),
                _ => {}
            },
        })?;

        if let Ok(event) = event::read() {
            match event {
                event::Event::Key(KeyEvent { code, .. }) => match app_state.input_mode {
                    InputMode::Normal => handle_normal_mode(&mut app_state, code, &running)?,
                    InputMode::Editing => handle_editing_mode(&mut app_state, code, &running)?,
                    InputMode::Viewing => handle_viewing_mode(&mut app_state, code, &running)?,
                    InputMode::EnterAddress => {
                        handle_enter_address_mode(&mut app_state, code, &running)?
                    }
                    InputMode::ViewResults => {
                        handle_view_results_mode(&mut app_state, code, &running)?
                    }
                },
                _ => {}
            }
        }

        // Sleep briefly to reduce CPU usage
        thread::sleep(Duration::from_millis(10));
    }

    Ok(())
}

// === Input Handling Functions ===

fn handle_normal_mode(
    app_state: &mut AppState,
    code: KeyCode,
    running: &Arc<AtomicBool>,
) -> io::Result<()> {
    match (code, &app_state.active_menu) {
        (KeyCode::Char('q'), _) => {
            running.store(false, Ordering::Relaxed);
        }
        (KeyCode::Char('1'), MenuItem::Main) => {
            app_state.active_menu = MenuItem::PasswordManager;
        }
        (KeyCode::Char('2'), MenuItem::Main) => {
            app_state.active_menu = MenuItem::NetworkTools;
        }
        (KeyCode::Char('a'), MenuItem::PasswordManager) => {
            app_state.input_mode = InputMode::Editing;
        }
        (KeyCode::Char('v'), MenuItem::PasswordManager) => {
            app_state.input_mode = InputMode::Viewing;
        }
        (KeyCode::Char('p'), MenuItem::NetworkTools) => {
            app_state.selected_tool = Some("ping".to_string());
            app_state.input_mode = InputMode::EnterAddress;
        }
        (KeyCode::Char('t'), MenuItem::NetworkTools) => {
            app_state.selected_tool = Some("traceroute".to_string());
            app_state.input_mode = InputMode::EnterAddress;
        }
        (KeyCode::Esc, menu_item) if *menu_item != MenuItem::Main => {
            app_state.active_menu = MenuItem::Main;
            app_state.input_mode = InputMode::Normal;
            app_state.selected_tool = None;
            app_state.address.clear();
            app_state.result = None;
            app_state.error_message = None;
        }
        _ => {}
    }
    Ok(())
}

fn handle_editing_mode(
    app_state: &mut AppState,
    code: KeyCode,
    running: &Arc<AtomicBool>,
) -> io::Result<()> {
    match code {
        KeyCode::Esc => {
            app_state.input_mode = InputMode::Normal;
            app_state.error_message = None;
        }
        KeyCode::Char('q') => {
            running.store(false, Ordering::Relaxed);
        }
        KeyCode::Tab => {
            app_state.input_field = (app_state.input_field + 1) % 3;
        }
        KeyCode::BackTab => {
            app_state.input_field = (app_state.input_field + 2) % 3;
        }
        KeyCode::Enter => {
            let entry = PasswordEntry {
                service: app_state.service.clone(),
                username: app_state.username.clone(),
                password: app_state.password.clone(),
            };

            if let Err(e) = save_password(&entry) {
                app_state.error_message = Some(format!("Error saving password: {}", e));
            } else {
                app_state.error_message = Some("Password saved successfully.".to_string());
                app_state.service.clear();
                app_state.username.clear();
                app_state.password.clear();
                app_state.input_mode = InputMode::Normal;
            }
        }
        KeyCode::Char(c) => match app_state.input_field {
            0 => app_state.service.push(c),
            1 => app_state.username.push(c),
            2 => app_state.password.push(c),
            _ => {}
        },
        KeyCode::Backspace => match app_state.input_field {
            0 => {
                app_state.service.pop();
            }
            1 => {
                app_state.username.pop();
            }
            2 => {
                app_state.password.pop();
            }
            _ => {}
        },
        _ => {}
    }
    Ok(())
}

fn handle_viewing_mode(
    app_state: &mut AppState,
    code: KeyCode,
    running: &Arc<AtomicBool>,
) -> io::Result<()> {
    match code {
        KeyCode::Esc | KeyCode::Enter => {
            app_state.input_mode = InputMode::Normal;
        }
        KeyCode::Char('q') => {
            running.store(false, Ordering::Relaxed);
        }
        _ => {}
    }
    Ok(())
}

fn handle_enter_address_mode(
    app_state: &mut AppState,
    code: KeyCode,
    running: &Arc<AtomicBool>,
) -> io::Result<()> {
    match code {
        KeyCode::Esc => {
            app_state.input_mode = InputMode::Normal;
            app_state.address.clear();
            app_state.selected_tool = None;
        }
        KeyCode::Char('q') => {
            running.store(false, Ordering::Relaxed);
        }
        KeyCode::Enter => {
            if let Some(ref tool) = app_state.selected_tool {
                let address = app_state.address.trim();
                let result = match tool.as_str() {
                    "ping" => ping(address).map(|output| {
                        // Store the PingResult as a JSON string for simplicity
                        serde_json::to_string(&output).unwrap_or_else(|_| {
                            "Failed to serialize ping result.".to_string()
                        })
                    }),
                    // Handle other tools if necessary
                    _ => Err("Unsupported tool".into()),
                };

                match result {
                    Ok(json_result) => {
                        app_state.result = Some(json_result);
                        app_state.input_mode = InputMode::ViewResults;
                    }
                    Err(e) => {
                        println!("Failed to parse packet stats: {:?}", e);
                        app_state.error_message = Some(format!("Error: {}", e));
                        app_state.input_mode = InputMode::Normal;
                    }
                }
                app_state.address.clear();
            } else {
                app_state.error_message = Some("No tool selected.".to_string());
                app_state.input_mode = InputMode::Normal;
            }
        }
        KeyCode::Char(c) => {
            app_state.address.push(c);
        }
        KeyCode::Backspace => {
            app_state.address.pop();
        }
        _ => {}
    }
    Ok(())
}

fn handle_view_results_mode(
    app_state: &mut AppState,
    code: KeyCode,
    running: &Arc<AtomicBool>,
) -> io::Result<()> {
    match code {
        KeyCode::Esc | KeyCode::Enter => {
            app_state.input_mode = InputMode::Normal;
            app_state.result = None;
            app_state.selected_tool = None;
        }
        KeyCode::Char('q') => {
            running.store(false, Ordering::Relaxed);
        }
        _ => {}
    }
    Ok(())
}

// === Drawing Functions ===

fn is_dark_mode() -> bool {
    let output = Command::new("osascript")
        .arg("-e")
        .arg("tell application \"System Events\" to tell appearance preferences to return dark mode")
        .output()
        .expect("Failed to execute osascript");

    let result = String::from_utf8_lossy(&output.stdout);
    result.trim() == "true"
}

fn get_text_color() -> Color {
    if is_dark_mode() {
        Color::White
    } else {
        Color::Black
    }
}

fn draw_main_menu<B: Backend>(f: &mut Frame<B>) {
    let text_color = get_text_color();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(f.size());

    let text = vec![
        Spans::from(Span::raw("1. Password Manager")),
        Spans::from(Span::raw("2. Network Tools")),
        Spans::from(Span::raw("Press 'q' to quit")),
    ];
    let paragraph = Paragraph::new(text)
        .block(Block::default().title("Main Menu").borders(Borders::ALL))
        .style(Style::default().fg(text_color)); // Use dynamic text color
    f.render_widget(paragraph, chunks[0]);
}

fn draw_password_manager_menu<B: Backend>(f: &mut Frame<B>) {
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
        .style(Style::default().fg(Color::Black)); // Set text color to black
    f.render_widget(paragraph, chunks[0]);
}

fn draw_input_modal<B: Backend>(f: &mut Frame<B>, app_state: &AppState) {
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

    let fields = vec![
        ("Service: ", &app_state.service, app_state.input_field == 0),
        ("Username: ", &app_state.username, app_state.input_field == 1),
        ("Password: ", &app_state.password, app_state.input_field == 2),
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

fn draw_password_list<B: Backend>(f: &mut Frame<B>) {
    match password_manager::retrieve_password() {
        Ok(passwords) => {
            if passwords.is_empty() {
                let paragraph = Paragraph::new("No passwords found.")
                    .block(Block::default().title("Stored Passwords").borders(Borders::ALL))
                    .style(Style::default().fg(Color::Black)); // Set text color to black
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
                    .block(Block::default().title("Stored Passwords").borders(Borders::ALL))
                    .style(Style::default().fg(Color::Black)); // Set text color to black
                f.render_widget(paragraph, f.size());
            }
        }
        Err(e) => {
            let paragraph = Paragraph::new(format!("Error retrieving passwords: {:?}", e))
                .block(Block::default().title("Error").borders(Borders::ALL))
                .style(Style::default().fg(Color::Red)); // Keep error text red
            f.render_widget(paragraph, f.size());
        }
    }
}

fn draw_network_tools_menu<B: Backend>(f: &mut Frame<B>) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(f.size());

    let text = vec![
        Spans::from(Span::raw("p. Ping")),
        Spans::from(Span::raw("t. Traceroute")),
        Spans::from(Span::raw("Esc. Back to Main Menu")),
        Spans::from(Span::raw("Press 'q' to quit")),
    ];
    let paragraph = Paragraph::new(text)
        .block(Block::default().title("Network Tools").borders(Borders::ALL))
        .style(Style::default().fg(Color::Black)); // Set text color to black
    f.render_widget(paragraph, chunks[0]);
}

fn draw_address_input<B: Backend>(f: &mut Frame<B>, app_state: &AppState) {
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
        .block(Block::default().title("Address Input").borders(Borders::ALL))
        .style(Style::default().fg(Color::White)); // Remove bg setting
    f.render_widget(paragraph, chunks[0]);
}

use tui::widgets::{Table, Row, Cell};

fn draw_view_results<B: Backend>(f: &mut Frame<B>, app_state: &AppState) {
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

            rows.extend(vec![
                Row::new(vec![
                    Cell::from("Round-Trip Min"),
                    Cell::from(format!("{} ms", ping_result.round_trip_min)),
                ]),
                Row::new(vec![
                    Cell::from("Round-Trip Avg"),
                    Cell::from(format!("{} ms", ping_result.round_trip_avg)),
                ]),
                Row::new(vec![
                    Cell::from("Round-Trip Max"),
                    Cell::from(format!("{} ms", ping_result.round_trip_max)),
                ]),
                Row::new(vec![
                    Cell::from("Round-Trip Mdev"),
                    Cell::from(format!("{} ms", ping_result.round_trip_mdev)),
                ]),
            ]);

            let table = Table::new(rows)
                .header(Row::new(vec!["Metric", "Value"]).style(Style::default().fg(Color::Yellow)))
                .block(Block::default().title("Ping Results").borders(Borders::ALL))
                .widths(&[Constraint::Length(20), Constraint::Length(20)]);

            f.render_widget(table, chunks[0]);
        } else {
            // If deserialization fails, display the raw output
            let paragraph = Paragraph::new(result.clone())
                .block(Block::default().title("Results").borders(Borders::ALL))
                .style(Style::default().fg(Color::White)); // Remove bg setting
            f.render_widget(paragraph, f.size());
        }
    } else {
        let paragraph = Paragraph::new("No results available.")
            .block(Block::default().title("Results").borders(Borders::ALL))
            .style(Style::default().fg(Color::White)); // Remove bg setting
        f.render_widget(paragraph, f.size());
    }
}

#[allow(dead_code)]
fn update_ui_with_ping_result(result: String) {
    // Placeholder function to handle the ping result
    // You can update the app_state or UI here as needed
    println!("Ping result: {}", result);
}

#[allow(dead_code)]
fn update_ui_with_error(error_message: String) {
    // Placeholder function to handle errors
    // You can update the app_state or UI here as needed
    println!("Error: {}", error_message);
}
