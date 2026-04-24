use ratatui::{prelude::*, widgets::*};

use super::input::InputField;
use super::state::{App, Mode, StatusState};
use super::theme::*;

// ── Main render dispatcher ──

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();
    frame.render_widget(Block::default().style(Style::default().bg(BG)), area);

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
        Mode::Status(_) => {}
        Mode::Message { text, success } => {
            render_message_toast(frame, area, text, *success);
        }
    }
}

// ── Helpers ──

fn panel_block(title: &str) -> Block<'_> {
    Block::default()
        .title(format!(" {title} "))
        .title_style(Style::default().fg(ACCENT).bold())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(BORDER))
        .style(Style::default().bg(PANEL_BG))
}

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

// ── Main screens ──

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

fn render_input_modal(frame: &mut Frame, area: Rect, title: &str, label: &str, input: &InputField, step: u16, total: u16) {
    let r = centered_rect(60, 8, area);
    frame.render_widget(Clear, r);

    let block = modal_block(title, ACCENT);
    let inner = block.inner(r);
    frame.render_widget(block, r);

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

    let input_area = Rect::new(inner.x + 1, inner.y + 2, inner.width.saturating_sub(2), 3);
    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Rgb(70, 70, 100)))
        .style(Style::default().bg(Color::Rgb(20, 20, 35)));

    let input_inner = input_block.inner(input_area);
    frame.render_widget(input_block, input_area);

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
            Constraint::Length(3),
            Constraint::Length(5),
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .split(area);

    // Header
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

    // API info + Progress
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

    frame.render_widget(Paragraph::new(Line::from(vec![
        Span::styled(" API    ", Style::default().fg(DIM)),
        Span::styled(&state.api_url, Style::default().fg(TEXT)),
    ])), info_layout[0]);

    frame.render_widget(Paragraph::new(Line::from(vec![
        Span::styled(" Model  ", Style::default().fg(DIM)),
        Span::styled(&state.model, Style::default().fg(TEXT)),
    ])), info_layout[1]);

    let ratio = if state.total > 0 { state.checked as f64 / state.total as f64 } else { 0.0 };
    let gauge_label = format!("{}/{}", state.checked, state.total);
    let gauge = Gauge::default()
        .gauge_style(Style::default().fg(if done { SUCCESS } else { Color::Cyan }).bg(Color::Rgb(30, 30, 50)))
        .ratio(ratio)
        .label(Span::styled(gauge_label, Style::default().fg(Color::White).bold()));
    frame.render_widget(gauge, Rect::new(info_layout[2].x + 1, info_layout[2].y, info_layout[2].width.saturating_sub(2), 1));

    // Results table
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

    // Summary footer
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
            Span::styled(" Checking... ".to_string(), Style::default().fg(Color::Cyan)),
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
