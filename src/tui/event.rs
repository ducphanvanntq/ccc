use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::io::stdout;

use super::input::InputField;
use super::render::render;
use super::state::{App, Mode};

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
        app.poll_status();

        terminal.draw(|f| render(f, &app)).ok();
        if app.should_quit { break; }

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
