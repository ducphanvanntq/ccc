use std::sync::mpsc;

use super::input::InputField;

// ── App mode (state machine) ──

pub enum Mode {
    Normal,
    AddName(InputField),
    AddValue { name: String, input: InputField },
    Rename { old_name: String, input: InputField },
    ConfirmRemove(String),
    Status(StatusState),
    Message { text: String, success: bool },
}

pub struct StatusState {
    pub api_url: String,
    pub model: String,
    pub results: Vec<StatusEntry>,
    pub total: usize,
    pub checked: usize,
    pub rx: Option<mpsc::Receiver<(usize, bool, String)>>,
}

#[derive(Clone)]
pub struct StatusEntry {
    pub name: String,
    pub masked: String,
    pub is_default: bool,
    pub result: Option<(bool, String)>,
}

// ── App ──

pub struct App {
    pub entries: Vec<(String, String, bool)>,
    pub selected: usize,
    pub mode: Mode,
    pub should_quit: bool,
}
