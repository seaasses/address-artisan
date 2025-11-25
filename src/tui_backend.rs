use crate::display_backend::{BenchStats, UiBackend};
use crate::prefix::{AddressType, Prefix};
use crossterm::{
    cursor::Show,
    event::{self, Event, KeyCode, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, ListState, Paragraph, Row, Table, TableState, Wrap},
    Terminal,
};
use std::collections::HashMap;
use std::io::{self, stdout};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

// TUI Layout Constants
const EVENT_POLL_INTERVAL_MS: u64 = 250;
const EVENT_POLL_INTERVAL_EXIT_MS: u64 = 10; // Faster polling when exiting
const CONFIG_SECTION_HEIGHT: u16 = 5; // 3 lines of content + 2 for borders

// Table Column Widths
const WORKBENCH_ID_COLUMN_WIDTH: u16 = 30;
const WORKBENCH_ADDR_S_COLUMN_WIDTH: u16 = 12;
const WORKBENCH_GENERATED_COLUMN_WIDTH: u16 = 15;
const WORKBENCH_RUNTIME_COLUMN_WIDTH: u16 = 10;

const WORKBENCH_ID_MAX_DISPLAY_LEN: usize = 30;
const WORKBENCH_ID_TRUNCATE_LEN: usize = 27;

/// Formats a BIP32 derivation path for display
fn format_derivation_path(path: &[u32; 6]) -> String {
    format!("xpub'/{}/{}/{}/{}", path[0], path[1], path[2], path[3])
}

/// Gets all workbench IDs from both workbench_status and bench_stats, sorted
fn get_all_workbench_ids(
    workbench_status: &HashMap<String, WorkbenchStatus>,
    bench_stats: &HashMap<String, BenchStats>,
) -> Vec<String> {
    let mut all_ids: std::collections::HashSet<String> = workbench_status.keys().cloned().collect();
    for id in bench_stats.keys() {
        all_ids.insert(id.clone());
    }
    let mut sorted: Vec<_> = all_ids.into_iter().collect();
    sorted.sort();
    sorted
}

struct TuiState {
    config: Arc<ConfigInfo>,
    prefixes: Arc<Vec<Prefix>>,
    bench_stats: HashMap<String, BenchStats>,
    workbench_status: HashMap<String, WorkbenchStatus>,
    found_addresses: Vec<FoundAddress>,
    workbenches_list_state: ListState,
    found_list_state: ListState,
    active_list: ActiveList,
}

#[derive(Clone)]
enum WorkbenchStatus {
    Starting,
    Running,
    Stopping,
    Stopped,
}

impl WorkbenchStatus {
    fn as_str(&self) -> &str {
        match self {
            WorkbenchStatus::Starting => "Starting",
            WorkbenchStatus::Running => "Running",
            WorkbenchStatus::Stopping => "Stopping",
            WorkbenchStatus::Stopped => "Stopped",
        }
    }
}

#[derive(PartialEq)]
enum ActiveList {
    Workbenches,
    Found,
}

#[derive(Clone)]
struct ConfigInfo {
    max_depth: u32,
    cpu_threads: u32,
}

#[derive(Clone)]
struct FoundAddress {
    address: String,
    bench_id: String,
    path: [u32; 6],
    prefix_id: u8,
}

pub struct TuiBackend {
    state: Arc<Mutex<TuiState>>,
    stop_signal: Arc<AtomicBool>,
    event_thread: Option<JoinHandle<()>>,
    render_requested: Arc<AtomicBool>,
    exit_requested: Arc<AtomicBool>,
}

impl TuiBackend {
    pub fn new(stop_signal: Arc<AtomicBool>) -> io::Result<Self> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = stdout();
        stdout.execute(EnterAlternateScreen)?;

        let mut workbenches_list_state = ListState::default();
        workbenches_list_state.select(Some(0));

        let mut found_list_state = ListState::default();
        found_list_state.select(Some(0));

        let state = Arc::new(Mutex::new(TuiState {
            config: Arc::new(ConfigInfo {
                max_depth: 0,
                cpu_threads: 0,
            }),
            prefixes: Arc::new(Vec::new()),
            bench_stats: HashMap::new(),
            workbench_status: HashMap::new(),
            found_addresses: Vec::new(),
            workbenches_list_state,
            found_list_state,
            active_list: ActiveList::Workbenches,
        }));

        let render_requested = Arc::new(AtomicBool::new(true)); // Initial render
        let exit_requested = Arc::new(AtomicBool::new(false));

        // Spawn event loop thread
        let state_clone = Arc::clone(&state);
        let stop_signal_clone = Arc::clone(&stop_signal);
        let render_requested_clone = Arc::clone(&render_requested);
        let exit_requested_clone = Arc::clone(&exit_requested);

        let event_thread = thread::spawn(move || {
            if let Err(e) = run_event_loop(
                state_clone,
                stop_signal_clone,
                render_requested_clone,
                exit_requested_clone,
            ) {
                eprintln!("TUI event loop error: {}", e);
            }
        });

        Ok(TuiBackend {
            state,
            stop_signal,
            event_thread: Some(event_thread),
            render_requested,
            exit_requested,
        })
    }
}

impl UiBackend for TuiBackend {
    fn start(&mut self, prefixes: &[Prefix], max_depth: u32, cpu_threads: u32) {
        let mut state = self.state.lock().expect("TUI state mutex poisoned");
        state.config = Arc::new(ConfigInfo {
            max_depth,
            cpu_threads,
        });
        state.prefixes = Arc::new(prefixes.to_vec());
        drop(state);
        self.render_requested.store(true, Ordering::Relaxed);
    }

    fn workbench_starting(&mut self, bench_id: &str) {
        let mut state = self.state.lock().expect("TUI state mutex poisoned");
        state
            .workbench_status
            .insert(bench_id.to_string(), WorkbenchStatus::Starting);
        drop(state);
        self.render_requested.store(true, Ordering::Relaxed);
    }

    fn workbench_started(&mut self, bench_id: &str) {
        let mut state = self.state.lock().expect("TUI state mutex poisoned");
        state
            .workbench_status
            .insert(bench_id.to_string(), WorkbenchStatus::Running);
        drop(state);
        self.render_requested.store(true, Ordering::Relaxed);
    }

    fn log_status(&mut self, bench_stats: &HashMap<String, BenchStats>) {
        let mut state = self.state.lock().expect("TUI state mutex poisoned");
        state.bench_stats = bench_stats.clone();
        drop(state); // Release lock before setting flag
        self.render_requested.store(true, Ordering::Relaxed);
    }

    fn log_found_address(&mut self, bench_id: &str, address: &str, path: &[u32; 6], prefix_id: u8) {
        let mut state = self.state.lock().expect("TUI state mutex poisoned");
        state.found_addresses.push(FoundAddress {
            address: address.to_string(),
            bench_id: bench_id.to_string(),
            path: *path,
            prefix_id,
        });
        drop(state);
        self.render_requested.store(true, Ordering::Relaxed);
    }

    fn log_derivation_error(&mut self) {
        // Could add to an errors list in TuiState if needed
    }

    fn log_false_positive(&mut self, _bench_id: &str, _path: &[u32; 6]) {
        // Could add to a false positives list if needed
    }

    fn stop_requested(&mut self) {
        // TUI will display this via the stop_signal being set
    }

    fn workbench_stopping(&mut self, bench_id: &str) {
        let mut state = self.state.lock().expect("TUI state mutex poisoned");
        state
            .workbench_status
            .insert(bench_id.to_string(), WorkbenchStatus::Stopping);
        drop(state);
        self.render_requested.store(true, Ordering::Relaxed);
    }

    fn workbench_stopped(&mut self, bench_id: &str, _total_generated: u64, _elapsed: Duration) {
        let mut state = self.state.lock().expect("TUI state mutex poisoned");
        state
            .workbench_status
            .insert(bench_id.to_string(), WorkbenchStatus::Stopped);
        drop(state);
        self.render_requested.store(true, Ordering::Relaxed);
    }

    fn final_status(&mut self) {
        // Signal the event loop to exit - all workbenches have stopped
        self.exit_requested.store(true, Ordering::Relaxed);
    }
}

impl Drop for TuiBackend {
    fn drop(&mut self) {
        // Signal event thread to stop
        self.stop_signal.store(true, Ordering::Relaxed);

        // Wait for event thread to finish
        if let Some(handle) = self.event_thread.take() {
            let _ = handle.join();
        }

        // Cleanup terminal
        let _ = disable_raw_mode();
        let _ = stdout().execute(LeaveAlternateScreen);
        let _ = stdout().execute(Show);

        // Print found addresses after exiting TUI in CSV format
        let state = self.state.lock().expect("TUI state mutex poisoned");
        if !state.found_addresses.is_empty() {
            println!("\naddress,type,prefix,derivation path,index");
            for item in &state.found_addresses {
                let derivation_path = format_derivation_path(&item.path);
                let prefix = &state.prefixes[item.prefix_id as usize];
                let prefix_str = prefix.as_str();
                let address_type_str = match prefix.address_type {
                    AddressType::P2PKH => "P2PKH",
                    AddressType::P2WPKH => "P2WPKH",
                };
                println!(
                    "{},{},{},{},{}",
                    item.address, address_type_str, prefix_str, derivation_path, item.path[5]
                );
            }
        }
    }
}

fn run_event_loop(
    state: Arc<Mutex<TuiState>>,
    stop_signal: Arc<AtomicBool>,
    render_requested: Arc<AtomicBool>,
    exit_requested: Arc<AtomicBool>,
) -> io::Result<()> {
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    loop {
        // Exit when orchestrator calls final_status()
        if exit_requested.load(Ordering::Relaxed) {
            break;
        }

        // Only render when orchestrator requests it (via log_status)
        if render_requested.swap(false, Ordering::Relaxed) {
            // Quick lock to copy data, then unlock before rendering
            let (
                config,
                prefixes,
                bench_stats,
                workbench_status,
                found_addresses,
                workbenches_selected,
                found_selected,
                active_list,
            ) = {
                let state = state.lock().expect("TUI state mutex poisoned");
                (
                    state.config.clone(),
                    state.prefixes.clone(),
                    state.bench_stats.clone(),
                    state.workbench_status.clone(),
                    state.found_addresses.clone(),
                    state.workbenches_list_state.selected(),
                    state.found_list_state.selected(),
                    state.active_list == ActiveList::Workbenches,
                )
            };
            // Lock is released here - workbenches can now update freely

            // Render UI using copied data (no lock held)
            terminal.draw(|frame| {
                // Calculate exact layout heights
                let config_height = CONFIG_SECTION_HEIGHT;

                // Workbenches: header(1) + data rows + totals(1) + borders(2)
                // Count both starting and running workbenches
                let all_bench_ids = get_all_workbench_ids(&workbench_status, &bench_stats);
                let num_workbenches = all_bench_ids.len().max(1); // At least show 1 row for layout
                let workbenches_height = 1 + num_workbenches as u16 + 1 + 2;

                let chunks = Layout::vertical([
                    Constraint::Length(2),                  // Title
                    Constraint::Length(config_height),      // Configuration
                    Constraint::Length(workbenches_height), // Workbenches (exact size)
                    Constraint::Fill(1),                    // Found addresses (takes remaining)
                    Constraint::Length(1),                  // Instructions
                ])
                .split(frame.area());

                // Title
                let title = Paragraph::new("Address Artisan")
                    .style(Style::default().add_modifier(Modifier::BOLD));
                frame.render_widget(title, chunks[0]);

                // Configuration
                let prefix_label = if prefixes.len() == 1 {
                    "Prefix"
                } else {
                    "Prefixes"
                };
                let prefixes_str = prefixes
                    .iter()
                    .map(|p| p.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");

                let config_lines = vec![
                    Line::from(vec![
                        Span::styled(prefix_label, Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(format!(": {}", prefixes_str)),
                    ]),
                    Line::from(vec![
                        Span::styled("Max depth", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(format!(": {}", config.max_depth)),
                    ]),
                    Line::from(vec![
                        Span::styled("CPU threads", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(format!(": {}", config.cpu_threads)),
                    ]),
                ];
                let config_widget = Paragraph::new(config_lines)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Configuration"),
                    )
                    .wrap(Wrap { trim: false });
                frame.render_widget(config_widget, chunks[1]);

                // Workbenches - using Table for proper column alignment
                let mut total_generated = 0u64;
                let mut total_hashrate = 0.0;

                // Collect all workbench IDs (both starting and running)
                let all_ids = get_all_workbench_ids(&workbench_status, &bench_stats);

                let mut workbench_rows: Vec<Row> = Vec::new();
                for bench_id in &all_ids {
                    let status = workbench_status
                        .get(bench_id)
                        .cloned()
                        .unwrap_or(WorkbenchStatus::Running);
                    let status_str = status.as_str();

                    // Truncate bench_id if longer than max display length
                    let display_id = if bench_id.len() > WORKBENCH_ID_MAX_DISPLAY_LEN {
                        format!("{}...", &bench_id[..WORKBENCH_ID_TRUNCATE_LEN])
                    } else {
                        bench_id.to_string()
                    };

                    if let Some(stats) = bench_stats.get(bench_id) {
                        // Has stats - show them
                        let runtime = stats.runtime_secs();
                        let hashrate = if runtime > 0 {
                            stats.total_generated / runtime
                        } else {
                            0
                        };

                        total_generated += stats.total_generated;
                        total_hashrate += hashrate as f64;

                        workbench_rows.push(Row::new(vec![
                            display_id,
                            hashrate.to_string(),
                            stats.total_generated.to_string(),
                            format!("{}s", runtime),
                            status_str.to_string(),
                        ]));
                    } else {
                        // No stats yet - just show status
                        workbench_rows.push(Row::new(vec![
                            display_id,
                            "-".to_string(),
                            "-".to_string(),
                            "-".to_string(),
                            status_str.to_string(),
                        ]));
                    }
                }

                // Add totals row
                workbench_rows.push(
                    Row::new(vec![
                        "TOTAL".to_string(),
                        (total_hashrate as u64).to_string(),
                        total_generated.to_string(),
                        "-".to_string(),
                        "-".to_string(),
                    ])
                    .style(Style::default().add_modifier(Modifier::BOLD)),
                );

                let workbenches_table = Table::new(
                    workbench_rows,
                    [
                        Constraint::Length(WORKBENCH_ID_COLUMN_WIDTH),     // ID
                        Constraint::Length(WORKBENCH_ADDR_S_COLUMN_WIDTH), // Addr/s
                        Constraint::Length(WORKBENCH_GENERATED_COLUMN_WIDTH), // Generated
                        Constraint::Length(WORKBENCH_RUNTIME_COLUMN_WIDTH), // Runtime
                        Constraint::Min(10),                               // Status
                    ],
                )
                .header(
                    Row::new(vec!["ID", "Addr/s", "Generated", "Runtime", "Status"])
                        .style(Style::default().add_modifier(Modifier::BOLD)),
                )
                .block(Block::default().borders(Borders::ALL).title("Workbenches"))
                .column_spacing(2)
                .row_highlight_style(if active_list {
                    Style::default().add_modifier(Modifier::REVERSED)
                } else {
                    Style::default()
                });

                let mut workbenches_table_state = TableState::default();
                workbenches_table_state.select(workbenches_selected);
                frame.render_stateful_widget(
                    workbenches_table,
                    chunks[2],
                    &mut workbenches_table_state,
                );

                // Found addresses - using Table for proper column alignment
                // Calculate dynamic column widths and build rows in a single pass
                let (
                    rows,
                    max_address_len,
                    max_type_len,
                    max_prefix_len,
                    max_derivation_len,
                    max_index_len,
                ) = found_addresses.iter().fold(
                    (Vec::new(), 7, 4, 6, 15, 5), // Initial: (rows, addr, type, prefix, derivation, index)
                    |(
                        mut rows,
                        mut max_addr,
                        mut max_type,
                        mut max_pref,
                        mut max_deriv,
                        mut max_idx,
                    ),
                     item| {
                        let derivation_path = format_derivation_path(&item.path);
                        let index_str = item.path[5].to_string();
                        let prefix = &prefixes[item.prefix_id as usize];
                        let prefix_str = prefix.as_str().to_string();
                        let address_type_str = match prefix.address_type {
                            AddressType::P2PKH => "P2PKH",
                            AddressType::P2WPKH => "P2WPKH",
                        };

                        // Update max lengths
                        max_addr = max_addr.max(item.address.len());
                        max_type = max_type.max(address_type_str.len());
                        max_pref = max_pref.max(prefix_str.len());
                        max_deriv = max_deriv.max(derivation_path.len());
                        max_idx = max_idx.max(index_str.len());

                        rows.push(Row::new(vec![
                            item.address.clone(),
                            address_type_str.to_string(),
                            prefix_str,
                            derivation_path,
                            index_str,
                            item.bench_id.clone(),
                        ]));

                        (rows, max_addr, max_type, max_pref, max_deriv, max_idx)
                    },
                );

                let found_title = format!("Found Addresses ({})", found_addresses.len());
                let found_table = Table::new(
                    rows,
                    [
                        Constraint::Max(max_address_len as u16), // Address (max size, can be cut)
                        Constraint::Length(max_type_len as u16), // Type (P2PKH/P2WPKH - exact size)
                        Constraint::Max(max_prefix_len as u16),  // Prefix (max size, can be cut)
                        Constraint::Length(max_derivation_len as u16), // Derivation Path (EXACT size - highest priority)
                        Constraint::Length(max_index_len as u16), // Index (EXACT size - highest priority)
                        Constraint::Fill(1), // Found by (takes remaining space)
                    ],
                )
                .header(
                    Row::new(vec![
                        "Address",
                        "Type",
                        "Prefix",
                        "Derivation Path",
                        "Index",
                        "Found by",
                    ])
                    .style(Style::default().add_modifier(Modifier::BOLD)),
                )
                .block(Block::default().borders(Borders::ALL).title(found_title))
                .column_spacing(2)
                .row_highlight_style(if !active_list {
                    Style::default().add_modifier(Modifier::REVERSED)
                } else {
                    Style::default()
                });

                let mut found_table_state = TableState::default();
                found_table_state.select(found_selected);
                frame.render_stateful_widget(found_table, chunks[3], &mut found_table_state);

                // Instructions at the bottom
                let instructions = Paragraph::new(
                    "Tab: Switch between Workbenches/Found | ↑↓: Navigate | Ctrl+C: Stop",
                );
                frame.render_widget(instructions, chunks[4]);
            })?;
        }

        // Poll for events - use shorter interval when exiting for better responsiveness
        let poll_interval = if exit_requested.load(Ordering::Relaxed) {
            Duration::from_millis(EVENT_POLL_INTERVAL_EXIT_MS)
        } else {
            Duration::from_millis(EVENT_POLL_INTERVAL_MS)
        };

        if event::poll(poll_interval)? {
            match event::read()? {
                Event::Key(key) => {
                    match key.code {
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            // User pressed Ctrl+C - signal stop but keep rendering until all workbenches stop
                            stop_signal.store(true, Ordering::Relaxed);
                            render_requested.store(true, Ordering::Relaxed); // Force render to show stopping status
                        }
                        _ => {
                            // Handle other keys and request render only if state changed
                            let mut state = state.lock().expect("TUI state mutex poisoned");
                            let state_changed = handle_key_event(&mut state, key.code);
                            drop(state);
                            if state_changed {
                                render_requested.store(true, Ordering::Relaxed);
                            }
                        }
                    }
                }
                Event::Resize(_, _) => {
                    // Terminal was resized - force a redraw
                    render_requested.store(true, Ordering::Relaxed);
                }
                _ => {
                    // Ignore other events (mouse, focus, paste, etc.)
                }
            }
        }
    }

    Ok(())
}

/// Handles keyboard input events and returns whether the state changed
fn handle_key_event(state: &mut TuiState, key_code: KeyCode) -> bool {
    match key_code {
        KeyCode::Tab => {
            // Switch active list - always changes state
            state.active_list = match state.active_list {
                ActiveList::Workbenches => {
                    // Ensure found list has selection if not empty
                    if !state.found_addresses.is_empty()
                        && state.found_list_state.selected().is_none()
                    {
                        state.found_list_state.select(Some(0));
                    }
                    ActiveList::Found
                }
                ActiveList::Found => {
                    // Ensure workbenches list has selection if not empty
                    let all_ids =
                        get_all_workbench_ids(&state.workbench_status, &state.bench_stats);
                    if !all_ids.is_empty() && state.workbenches_list_state.selected().is_none() {
                        state.workbenches_list_state.select(Some(0));
                    }
                    ActiveList::Workbenches
                }
            };
            true
        }
        KeyCode::Down => {
            // Navigate down - only changes if list is not empty
            match state.active_list {
                ActiveList::Workbenches => {
                    let all_ids =
                        get_all_workbench_ids(&state.workbench_status, &state.bench_stats);
                    let len = all_ids.len();
                    if len > 0 {
                        let i = match state.workbenches_list_state.selected() {
                            Some(i) if i >= len - 1 => 0,
                            Some(i) => i + 1,
                            None => 0,
                        };
                        state.workbenches_list_state.select(Some(i));
                        true
                    } else {
                        false
                    }
                }
                ActiveList::Found => {
                    let len = state.found_addresses.len();
                    if len > 0 {
                        let i = match state.found_list_state.selected() {
                            Some(i) if i >= len - 1 => 0,
                            Some(i) => i + 1,
                            None => 0,
                        };
                        state.found_list_state.select(Some(i));
                        true
                    } else {
                        false
                    }
                }
            }
        }
        KeyCode::Up => {
            // Navigate up - only changes if list is not empty
            match state.active_list {
                ActiveList::Workbenches => {
                    let all_ids =
                        get_all_workbench_ids(&state.workbench_status, &state.bench_stats);
                    let len = all_ids.len();
                    if len > 0 {
                        let i = match state.workbenches_list_state.selected() {
                            Some(0) => len - 1,
                            Some(i) => i - 1,
                            None => 0,
                        };
                        state.workbenches_list_state.select(Some(i));
                        true
                    } else {
                        false
                    }
                }
                ActiveList::Found => {
                    let len = state.found_addresses.len();
                    if len > 0 {
                        let i = match state.found_list_state.selected() {
                            Some(0) => len - 1,
                            Some(i) => i - 1,
                            None => 0,
                        };
                        state.found_list_state.select(Some(i));
                        true
                    } else {
                        false
                    }
                }
            }
        }
        _ => false, // Unknown key - no state change
    }
}
