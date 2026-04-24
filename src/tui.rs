use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::*};
use std::io::stdout;
use std::sync::mpsc;
use std::thread;

use console::Emoji;

use crate::config::{local_settings_path, read_json_or_default, write_json, KeysStore};
use crate::utils::{check_api_key, get_api_config, mask_key, validate_key_format};

// ── Cross-platform icons (emoji with ASCII fallback) ──

static ICON_KEY:   Emoji = Emoji("🔑 ", "(K) ");
static ICON_SEARCH:Emoji = Emoji("🔍 ", "(?) ");
static ICON_OK:    Emoji = Emoji("✅", "[OK]");
static ICON_FAIL:  Emoji = Emoji("❌", "[X]");
static ICON_WAIT:  Emoji = Emoji("⏳", "[..]");
static ICON_STAR:  Emoji = Emoji("★", "*");
static ICON_PLAY:  Emoji = Emoji("▶", ">");
static ICON_CHECK: Emoji = Emoji("✓", "+");
static ICON_CROSS: Emoji = Emoji("✗", "x");

// ── Theme colors ──

const BG: Color = Color::Rgb(15, 15, 25);
const PANEL_BG: Color = Color::Rgb(22, 22, 35);
const MODAL_BG: Color = Color::Rgb(28, 28, 48);
const BORDER: Color = Color::Rgb(50, 50, 75);
const ACCENT: Color = Color::Rgb(130, 190, 255);
const DIM: Color = Color::Rgb(100, 100, 120);
const TEXT: Color = Color::Rgb(180, 180, 200);
const TEXT_DIM: Color = Color::Rgb(120, 120, 140);
const HIGHLIGHT_BG: Color = Color::Rgb(35, 40, 65);
const SUCCESS: Color = Color::Rgb(80, 220, 120);
const ERROR: Color = Color::Rgb(255, 100, 100);

// ── Input field ──

struct InputField {
    value: String,
    cursor: usize,
}

impl InputField {
    fn new() -> Self {
        Self { value: String::new(), cursor: 0 }
    }

    fn handle_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char(c) => {
                self.value.insert(self.cursor, c);
                self.cursor += 1;
            }
            KeyCode::Backspace => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                    self.value.remove(self.cursor);
                }
            }
            KeyCode::Left => {
                self.cursor = self.cursor.saturating_sub(1);
            }
            KeyCode::Right => {
                if self.cursor < self.value.len() {
                    self.cursor += 1;
                }
            }
            KeyCode::Home => self.cursor = 0,
            KeyCode::End => self.cursor = self.value.len(),
            _ => {}
        }
    }
}

// ── App mode (state machine) ──

enum Mode {
    Normal,
    AddName(InputField),
    AddValue { name: String, input: InputField },
    Rename { old_name: String, input: InputField },
    ConfirmRemove(String),
    Status(StatusState),
    Message { text: String, success: bool },
}

struct StatusState {
    api_url: String,
    model: String,
    results: Vec<StatusEntry>,
    total: usize,
    checked: usize,
    rx: Option<mpsc::Receiver<(usize, bool, String)>>,
}

#[derive(Clone)]
struct StatusEntry {
    name: String,
    masked: String,
    is_default: bool,
    result: Option<(bool, String)>,
}

// ── App ──

struct App {
    entries: Vec<(String, String, bool)>,
    selected: usize,
    mode: Mode,
    should_quit: bool,
}

impl App {
    fn load() -> Self {
        let store = KeysStore::load();
        let entries = Self::build_entries(&store);
        App { entries, selected: 0, mode: Mode::Normal, should_quit: false }
    }

    fn build_entries(store: &KeysStore) -> Vec<(String, String, bool)> {
        store.keys.iter()
            .map(|(name, value)| {
                let is_default = store.active.as_deref() == Some(name.as_str());
                (name.clone(), mask_key(value), is_default)
            })
            .collect()
    }

    fn reload(&mut self) {
        let store = KeysStore::load();
        self.entries = Self::build_entries(&store);
        if self.entries.is_empty() {
            self.selected = 0;
        } else if self.selected >= self.entries.len() {
            self.selected = self.entries.len() - 1;
        }
    }

    fn selected_name(&self) -> Option<String> {
        self.entries.get(self.selected).map(|(n, _, _)| n.clone())
    }

    fn msg(&mut self, text: String, success: bool) {
        self.mode = Mode::Message { text, success };
    }

    // ── Operations ──

    fn do_add(&mut self, name: &str, value: &str) {
        if name.is_empty() {
            self.msg("Name cannot be empty.".into(), false);
            return;
        }
        if let Err(e) = validate_key_format(value) {
            self.msg(e, false);
            return;
        }
        let mut store = KeysStore::load();
        let is_first = store.active.is_none();
        store.keys.insert(name.to_string(), value.to_string());
        if is_first {
            store.active = Some(name.to_string());
        }
        if let Err(e) = store.save() {
            self.msg(format!("Failed to save: {e}"), false);
            return;
        }
        if is_first {
            self.msg(format!("{} Added '{name}' and set as default.", ICON_CHECK), true);
        } else {
            self.msg(format!("{} Added '{name}'.", ICON_CHECK), true);
        }
        self.reload();
    }

    fn do_default(&mut self, name: &str) {
        let mut store = KeysStore::load();
        if !store.keys.contains_key(name) {
            self.msg(format!("Key '{name}' not found."), false);
            return;
        }
        store.active = Some(name.to_string());
        if let Err(e) = store.save() {
            self.msg(format!("Failed to save: {e}"), false);
            return;
        }
        self.msg(format!("{} Set '{name}' as default.", ICON_CHECK), true);
        self.reload();
    }

    fn do_use(&mut self, name: &str) {
        let store = KeysStore::load();
        let key_value = match store.keys.get(name) {
            Some(v) => v.clone(),
            None => { self.msg(format!("Key '{name}' not found."), false); return; }
        };
        let local_path = local_settings_path();
        if let Some(parent) = local_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let mut json = if local_path.exists() {
            read_json_or_default(&local_path)
        } else {
            serde_json::json!({})
        };
        json["env"]["ANTHROPIC_API_KEY"] = serde_json::Value::String(key_value);
        if let Err(e) = write_json(&local_path, &json) {
            self.msg(format!("Failed to write config: {e}"), false);
            return;
        }
        let folder = std::env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| ".".into());
        self.msg(format!("{} Using '{name}' for {folder}", ICON_CHECK), true);
    }

    fn do_remove(&mut self, name: &str) {
        let mut store = KeysStore::load();
        store.keys.remove(name);
        if store.active.as_deref() == Some(name) {
            store.active = store.keys.keys().next().cloned();
        }
        if let Err(e) = store.save() {
            self.msg(format!("Failed to save: {e}"), false);
            return;
        }
        self.msg(format!("{} Removed '{name}'.", ICON_CHECK), true);
        self.reload();
    }

    fn do_rename(&mut self, old_name: &str, new_name: &str) {
        if new_name.is_empty() {
            self.msg("Name cannot be empty.".into(), false);
            return;
        }
        if new_name == old_name {
            self.mode = Mode::Normal;
            return;
        }
        let mut store = KeysStore::load();
        if store.keys.contains_key(new_name) {
            self.msg(format!("Key '{new_name}' already exists."), false);
            return;
        }
        if let Some(value) = store.keys.remove(old_name) {
            store.keys.insert(new_name.to_string(), value);
            if store.active.as_deref() == Some(old_name) {
                store.active = Some(new_name.to_string());
            }
            if let Err(e) = store.save() {
                self.msg(format!("Failed to save: {e}"), false);
                return;
            }
            self.msg(format!("{} Renamed '{old_name}' → '{new_name}'.", ICON_CHECK), true);
            self.reload();
        }
    }

    fn do_status(&mut self) {
        let store = KeysStore::load();
        if store.keys.is_empty() {
            self.msg("No keys to check.".into(), false);
            return;
        }

        let (api_url, model) = get_api_config();
        let total = store.keys.len();

        let results: Vec<StatusEntry> = store.keys.iter()
            .map(|(name, value)| StatusEntry {
                name: name.clone(),
                masked: mask_key(value),
                is_default: store.active.as_deref() == Some(name.as_str()),
                result: None,
            })
            .collect();

        // Spawn threads for parallel key checking
        let (tx, rx) = mpsc::channel();
        for (i, (_, value)) in store.keys.iter().enumerate() {
            let tx = tx.clone();
            let value = value.clone();
            thread::spawn(move || {
                let (ok, msg) = check_api_key(&value);
                let _ = tx.send((i, ok, if ok { "OK".into() } else { msg }));
            });
        }
        drop(tx); // Drop sender so rx knows when all threads are done

        self.mode = Mode::Status(StatusState {
            api_url,
            model,
            results,
            total,
            checked: 0,
            rx: Some(rx),
        });
    }

    /// Poll for completed status checks (non-blocking)
    fn poll_status(&mut self) {
        if let Mode::Status(state) = &mut self.mode {
            if let Some(rx) = &state.rx {
                // Drain all available results
                while let Ok((idx, ok, msg)) = rx.try_recv() {
                    if idx < state.results.len() {
                        state.results[idx].result = Some((ok, msg));
                        state.checked += 1;
                    }
                }
                // If all done, drop the receiver
                if state.checked >= state.total {
                    state.rx = None;
                }
            }
        }
    }
}

// ── Main TUI entry ──

pub fn run_key_tui() {
    if enable_raw_mode().is_err() { return; }
    let mut stdout = stdout();
    if execute!(stdout, EnterAlternateScreen).is_err() {
        disable_raw_mode().ok();
        return;
    }

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = match Terminal::new(backend) {
        Ok(t) => t,
        Err(_) => { disable_raw_mode().ok(); return; }
    };

    let mut app = App::load();

    loop {
        // Poll for async status results before drawing
        app.poll_status();

        terminal.draw(|f| render(f, &app)).ok();
        if app.should_quit { break; }

        // Use poll with timeout so status updates render without waiting for keypress
        let has_pending_status = matches!(&app.mode, Mode::Status(s) if s.rx.is_some());
        let timeout = if has_pending_status {
            std::time::Duration::from_millis(50)
        } else {
            std::time::Duration::from_secs(60)
        };

        if !event::poll(timeout).unwrap_or(false) {
            continue;
        }

        let event = match event::read() {
            Ok(e) => e,
            Err(_) => continue,
        };

        if let Event::Key(key) = event {
            if key.kind != KeyEventKind::Press { continue; }

            match &app.mode {
                Mode::Normal => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
                    KeyCode::Char('a') => app.mode = Mode::AddName(InputField::new()),
                    KeyCode::Char('d') => {
                        if let Some(name) = app.selected_name() { app.do_default(&name); }
                    }
                    KeyCode::Char('u') => {
                        if let Some(name) = app.selected_name() { app.do_use(&name); }
                    }
                    KeyCode::Char('r') => {
                        if let Some(name) = app.selected_name() {
                            app.mode = Mode::ConfirmRemove(name);
                        }
                    }
                    KeyCode::Char('n') => {
                        if let Some(name) = app.selected_name() {
                            app.mode = Mode::Rename { old_name: name, input: InputField::new() };
                        }
                    }
                    KeyCode::Char('s') => app.do_status(),
                    KeyCode::Up | KeyCode::Char('k') => {
                        if app.selected > 0 { app.selected -= 1; }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        let max = app.entries.len().saturating_sub(1);
                        if app.selected < max { app.selected += 1; }
                    }
                    _ => {}
                },

                Mode::AddName(_) | Mode::AddValue { .. } | Mode::Rename { .. } => match key.code {
                    KeyCode::Esc => app.mode = Mode::Normal,
                    KeyCode::Enter => {
                        let mode = std::mem::replace(&mut app.mode, Mode::Normal);
                        match mode {
                            Mode::AddName(input) => {
                                if input.value.is_empty() {
                                    app.msg("Name cannot be empty.".into(), false);
                                } else {
                                    app.mode = Mode::AddValue { name: input.value, input: InputField::new() };
                                }
                            }
                            Mode::AddValue { name, input } => app.do_add(&name, &input.value),
                            Mode::Rename { old_name, input } => app.do_rename(&old_name, &input.value),
                            _ => {}
                        }
                    }
                    other => {
                        match &mut app.mode {
                            Mode::AddName(input)
                            | Mode::AddValue { input, .. }
                            | Mode::Rename { input, .. } => {
                                input.handle_key(other);
                            }
                            _ => {}
                        }
                    }
                },

                Mode::ConfirmRemove(_) => match key.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                        let mode = std::mem::replace(&mut app.mode, Mode::Normal);
                        if let Mode::ConfirmRemove(name) = mode {
                            app.do_remove(&name);
                        }
                    }
                    _ => app.mode = Mode::Normal,
                },

                Mode::Status(state) => {
                    // Only allow dismiss when all checks are done
                    if state.rx.is_none() {
                        app.mode = Mode::Normal;
                    }
                }
                Mode::Message { .. } => {
                    app.mode = Mode::Normal;
                }
            }
        }
    }

    disable_raw_mode().ok();
    execute!(terminal.backend_mut(), LeaveAlternateScreen).ok();
    terminal.show_cursor().ok();
}

// ── Rendering ──

fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();
    frame.render_widget(Block::default().style(Style::default().bg(BG)), area);

    // Status gets full-screen rendering
    if let Mode::Status(state) = &app.mode {
        render_status_screen(frame, area, state);
        return;
    }

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Length(3), Constraint::Min(5), Constraint::Length(3)])
        .split(area);

    render_header(frame, layout[0], app.entries.len());
    render_table(frame, layout[1], app);
    render_footer(frame, layout[2], &app.mode);

    // Modal overlay
    match &app.mode {
        Mode::Normal => {}
        Mode::AddName(input) => {
            render_input_modal(frame, area, "Add Key", "Key name (e.g. work, personal):", input, 1, 2);
        }
        Mode::AddValue { name, input } => {
            render_input_modal(frame, area, &format!("Add Key — '{name}'"), "API key value:", input, 2, 2);
        }
        Mode::Rename { old_name, input } => {
            render_input_modal(frame, area, &format!("Rename '{old_name}'"), "New name:", input, 1, 1);
        }
        Mode::ConfirmRemove(name) => {
            render_confirm_modal(frame, area, &format!("Remove '{name}'?"));
        }
        Mode::Status(_) => {} // handled above
        Mode::Message { text, success } => {
            render_message_toast(frame, area, text, *success);
        }
    }
}

fn panel_block(title: &str) -> Block<'_> {
    Block::default()
        .title(format!(" {title} "))
        .title_style(Style::default().fg(ACCENT).bold())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(BORDER))
        .style(Style::default().bg(PANEL_BG))
}

fn render_header(frame: &mut Frame, area: Rect, total: usize) {
    let title = Line::from(vec![
        Span::styled(format!("  {} ", ICON_KEY), Style::default().fg(Color::Yellow)),
        Span::styled("Key Manager", Style::default().fg(ACCENT).bold()),
        Span::styled(format!("  ({total} keys)"), Style::default().fg(DIM)),
    ]);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(BORDER))
        .style(Style::default().bg(PANEL_BG));
    frame.render_widget(Paragraph::new(title).block(block).alignment(Alignment::Center), area);
}

fn render_table(frame: &mut Frame, area: Rect, app: &App) {
    let block = panel_block("Keys");

    if app.entries.is_empty() {
        let empty = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled("No keys saved", Style::default().fg(DIM).italic())),
            Line::from(""),
            Line::from(vec![
                Span::styled("Press ", Style::default().fg(TEXT_DIM)),
                Span::styled("a", Style::default().fg(Color::Green).bold()),
                Span::styled(" to add your first key", Style::default().fg(TEXT_DIM)),
            ]),
        ])
        .alignment(Alignment::Center)
        .block(block);
        frame.render_widget(empty, area);
        return;
    }

    let header = Row::new(vec!["", "  Name", "Key", "Default"])
        .style(Style::default().fg(ACCENT).bold())
        .bottom_margin(1);

    let rows: Vec<Row> = app.entries.iter().enumerate().map(|(i, (name, masked, is_default))| {
        let sel = i == app.selected;
        let base = if sel {
            Style::default().bg(HIGHLIGHT_BG).fg(Color::White)
        } else {
            Style::default().fg(TEXT)
        };
        Row::new(vec![
            Cell::from(if sel { format!(" {ICON_PLAY}") } else { "  ".into() }).style(Style::default().fg(ACCENT)),
            Cell::from(format!("  {name}")),
            Cell::from(masked.as_str()).style(Style::default().fg(if sel { TEXT } else { TEXT_DIM })),
            Cell::from(if *is_default { format!(" {ICON_STAR}") } else { "".into() }).style(Style::default().fg(Color::Yellow)),
        ]).style(base)
    }).collect();

    let table = Table::new(rows, [
        Constraint::Length(3),
        Constraint::Percentage(35),
        Constraint::Percentage(45),
        Constraint::Length(10),
    ])
    .header(header)
    .block(block);
    frame.render_widget(table, area);
}

fn render_footer(frame: &mut Frame, area: Rect, mode: &Mode) {
    let line = match mode {
        Mode::AddName(_) | Mode::AddValue { .. } | Mode::Rename { .. } => Line::from(vec![
            Span::styled(" Enter", Style::default().fg(SUCCESS).bold()),
            Span::styled(" confirm  ", Style::default().fg(TEXT_DIM)),
            Span::styled("Esc", Style::default().fg(ERROR).bold()),
            Span::styled(" cancel ", Style::default().fg(TEXT_DIM)),
        ]),
        Mode::ConfirmRemove(_) => Line::from(vec![
            Span::styled(" [y]", Style::default().fg(ERROR).bold()),
            Span::styled("es  ", Style::default().fg(TEXT_DIM)),
            Span::styled("any key", Style::default().fg(DIM).bold()),
            Span::styled(" cancel ", Style::default().fg(TEXT_DIM)),
        ]),
        Mode::Status { .. } | Mode::Message { .. } => Line::from(vec![
            Span::styled(" Press any key to continue ", Style::default().fg(TEXT_DIM)),
        ]),
        Mode::Normal => Line::from(vec![
            Span::styled(" [a]", Style::default().fg(Color::Green).bold()),
            Span::styled("dd  ", Style::default().fg(TEXT_DIM)),
            Span::styled("[d]", Style::default().fg(Color::Yellow).bold()),
            Span::styled("efault  ", Style::default().fg(TEXT_DIM)),
            Span::styled("[u]", Style::default().fg(Color::Rgb(100, 149, 237)).bold()),
            Span::styled("se  ", Style::default().fg(TEXT_DIM)),
            Span::styled("[r]", Style::default().fg(ERROR).bold()),
            Span::styled("emove  ", Style::default().fg(TEXT_DIM)),
            Span::styled("re[n]", Style::default().fg(Color::Magenta).bold()),
            Span::styled("ame  ", Style::default().fg(TEXT_DIM)),
            Span::styled("[s]", Style::default().fg(Color::Cyan).bold()),
            Span::styled("tatus  ", Style::default().fg(TEXT_DIM)),
            Span::styled("[q]", Style::default().fg(Color::Rgb(80, 80, 100)).bold()),
            Span::styled("uit ", Style::default().fg(TEXT_DIM)),
        ]),
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(BORDER))
        .style(Style::default().bg(PANEL_BG));
    frame.render_widget(Paragraph::new(line).alignment(Alignment::Center).block(block), area);
}

// ── Modals ──

fn centered_rect(w: u16, h: u16, area: Rect) -> Rect {
    let x = area.x + area.width.saturating_sub(w) / 2;
    let y = area.y + area.height.saturating_sub(h) / 2;
    Rect::new(x, y, w.min(area.width), h.min(area.height))
}

fn modal_block(title: &str, border_color: Color) -> Block<'_> {
    Block::default()
        .title(format!(" {title} "))
        .title_style(Style::default().fg(border_color).bold())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(MODAL_BG))
}

fn render_input_modal(frame: &mut Frame, area: Rect, title: &str, label: &str, input: &InputField, step: u16, total: u16) {
    let r = centered_rect(60, 8, area);
    frame.render_widget(Clear, r);

    let block = modal_block(title, ACCENT);
    let inner = block.inner(r);
    frame.render_widget(block, r);

    // Step indicator
    let step_text = format!("Step {step}/{total}");
    let step_line = Line::from(vec![
        Span::styled(label, Style::default().fg(TEXT_DIM)),
        Span::raw("  "),
        Span::styled(step_text, Style::default().fg(DIM).italic()),
    ]);
    frame.render_widget(
        Paragraph::new(step_line),
        Rect::new(inner.x + 1, inner.y, inner.width.saturating_sub(2), 1),
    );

    // Input box
    let input_area = Rect::new(inner.x + 1, inner.y + 2, inner.width.saturating_sub(2), 3);
    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Rgb(70, 70, 100)))
        .style(Style::default().bg(Color::Rgb(20, 20, 35)));

    let input_inner = input_block.inner(input_area);
    frame.render_widget(input_block, input_area);

    // Text with cursor
    let before: String = input.value.chars().take(input.cursor).collect();
    let cursor_char = input.value.chars().nth(input.cursor).unwrap_or(' ');
    let after: String = input.value.chars().skip(input.cursor + 1).collect();

    let input_line = Line::from(vec![
        Span::styled(&before, Style::default().fg(Color::White)),
        Span::styled(
            cursor_char.to_string(),
            Style::default().fg(Color::Black).bg(ACCENT),
        ),
        Span::styled(&after, Style::default().fg(Color::White)),
    ]);
    frame.render_widget(Paragraph::new(input_line), input_inner);
}

fn render_confirm_modal(frame: &mut Frame, area: Rect, message: &str) {
    let r = centered_rect(50, 6, area);
    frame.render_widget(Clear, r);

    let block = modal_block("Confirm", ERROR);
    let inner = block.inner(r);
    frame.render_widget(block, r);

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(message, Style::default().fg(Color::White).bold())),
        Line::from(vec![
            Span::styled("[y]", Style::default().fg(ERROR).bold()),
            Span::styled(" yes   ", Style::default().fg(TEXT_DIM)),
            Span::styled("any key", Style::default().fg(DIM)),
            Span::styled(" cancel", Style::default().fg(TEXT_DIM)),
        ]),
    ];
    frame.render_widget(Paragraph::new(lines).alignment(Alignment::Center), inner);
}

fn render_status_screen(frame: &mut Frame, area: Rect, state: &StatusState) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Length(5),  // API info + progress
            Constraint::Min(5),    // Results table
            Constraint::Length(3),  // Summary + footer
        ])
        .split(area);

    // ── Header ──
    let done = state.checked == state.total;
    let header_text = if done { "Key Status — Complete" } else { "Key Status — Checking..." };
    let header = Paragraph::new(Line::from(vec![
        Span::styled(format!("  {} ", ICON_SEARCH), Style::default().fg(Color::Cyan)),
        Span::styled(header_text, Style::default().fg(ACCENT).bold()),
    ]))
    .alignment(Alignment::Center)
    .block(Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(if done { SUCCESS } else { Color::Cyan }))
        .style(Style::default().bg(PANEL_BG)));
    frame.render_widget(header, layout[0]);

    // ── API info + Progress ──
    let info_block = Block::default()
        .title(" Connection Info ")
        .title_style(Style::default().fg(ACCENT).bold())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(BORDER))
        .style(Style::default().bg(PANEL_BG));
    let info_inner = info_block.inner(layout[1]);
    frame.render_widget(info_block, layout[1]);

    let info_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1), Constraint::Length(1)])
        .split(info_inner);

    // API URL
    frame.render_widget(Paragraph::new(Line::from(vec![
        Span::styled(" API    ", Style::default().fg(DIM)),
        Span::styled(&state.api_url, Style::default().fg(TEXT)),
    ])), info_layout[0]);

    // Model
    frame.render_widget(Paragraph::new(Line::from(vec![
        Span::styled(" Model  ", Style::default().fg(DIM)),
        Span::styled(&state.model, Style::default().fg(TEXT)),
    ])), info_layout[1]);

    // Progress gauge
    let ratio = if state.total > 0 { state.checked as f64 / state.total as f64 } else { 0.0 };
    let gauge_label = format!("{}/{}", state.checked, state.total);
    let gauge = Gauge::default()
        .gauge_style(Style::default().fg(if done { SUCCESS } else { Color::Cyan }).bg(Color::Rgb(30, 30, 50)))
        .ratio(ratio)
        .label(Span::styled(gauge_label, Style::default().fg(Color::White).bold()));
    frame.render_widget(gauge, Rect::new(info_layout[2].x + 1, info_layout[2].y, info_layout[2].width.saturating_sub(2), 1));

    // ── Results table ──
    let results_block = panel_block("Results");
    let results_inner = results_block.inner(layout[2]);
    frame.render_widget(results_block, layout[2]);

    let header_row = Row::new(vec!["", "  Name", "Key", "Default", "Status"])
        .style(Style::default().fg(ACCENT).bold())
        .bottom_margin(1);

    let rows: Vec<Row> = state.results.iter().map(|entry| {
        let (icon, status_text, status_style) = match &entry.result {
            None => (ICON_WAIT.to_string(), "checking...".to_string(), Style::default().fg(Color::Yellow)),
            Some((true, msg)) => (ICON_OK.to_string(), msg.clone(), Style::default().fg(SUCCESS)),
            Some((false, msg)) => (ICON_FAIL.to_string(), msg.clone(), Style::default().fg(ERROR)),
        };
        let row_style = match &entry.result {
            None => Style::default().fg(Color::Yellow),
            Some((true, _)) => Style::default().fg(TEXT),
            Some((false, _)) => Style::default().fg(Color::Rgb(200, 150, 150)),
        };
        Row::new(vec![
            Cell::from(format!(" {icon}")),
            Cell::from(format!("  {}", entry.name)),
            Cell::from(entry.masked.as_str()).style(Style::default().fg(TEXT_DIM)),
            Cell::from(if entry.is_default { format!(" {ICON_STAR}") } else { "".into() }).style(Style::default().fg(Color::Yellow)),
            Cell::from(status_text).style(status_style),
        ]).style(row_style)
    }).collect();

    let table = Table::new(rows, [
        Constraint::Length(4),
        Constraint::Percentage(22),
        Constraint::Percentage(28),
        Constraint::Length(8),
        Constraint::Percentage(35),
    ])
    .header(header_row);
    frame.render_widget(table, results_inner);

    // ── Summary footer ──
    let pass = state.results.iter().filter(|e| matches!(&e.result, Some((true, _)))).count();
    let fail = state.results.iter().filter(|e| matches!(&e.result, Some((false, _)))).count();
    let pending = state.results.iter().filter(|e| e.result.is_none()).count();

    let summary = if done {
        Line::from(vec![
            Span::styled(format!(" {} ", ICON_OK), Style::default().fg(SUCCESS)),
            Span::styled(format!("{pass} passed"), Style::default().fg(SUCCESS).bold()),
            Span::styled("   ", Style::default()),
            Span::styled(format!("{} ", ICON_FAIL), Style::default().fg(if fail > 0 { ERROR } else { DIM })),
            Span::styled(format!("{fail} failed"), Style::default().fg(if fail > 0 { ERROR } else { DIM }).bold()),
            Span::styled("   │   ", Style::default().fg(BORDER)),
            Span::styled("Press any key to return", Style::default().fg(TEXT_DIM)),
        ])
    } else {
        Line::from(vec![
            Span::styled(format!(" Checking... "), Style::default().fg(Color::Cyan)),
            Span::styled(format!("({pending} remaining)"), Style::default().fg(DIM)),
        ])
    };

    let footer_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(if done { BORDER } else { Color::Cyan }))
        .style(Style::default().bg(PANEL_BG));
    frame.render_widget(Paragraph::new(summary).alignment(Alignment::Center).block(footer_block), layout[3]);
}

fn render_message_toast(frame: &mut Frame, area: Rect, text: &str, success: bool) {
    let w = (text.len() as u16 + 6).min(area.width.saturating_sub(4));
    let r = centered_rect(w, 5, area);
    frame.render_widget(Clear, r);

    let color = if success { SUCCESS } else { ERROR };
    let icon = if success { ICON_CHECK.to_string() } else { ICON_CROSS.to_string() };
    let block = modal_block(if success { "Success" } else { "Error" }, color);
    let inner = block.inner(r);
    frame.render_widget(block, r);

    let line = Line::from(vec![
        Span::styled(format!("{icon} "), Style::default().fg(color).bold()),
        Span::styled(text, Style::default().fg(Color::White)),
    ]);
    frame.render_widget(
        Paragraph::new(vec![Line::from(""), line]).alignment(Alignment::Center),
        inner,
    );
}
