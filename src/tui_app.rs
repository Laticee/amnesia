use crate::mem_buffer::MemoryBuffer;
use crate::persistence;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use std::time::{Duration, Instant};
use zeroize::Zeroize;

#[derive(PartialEq)]
pub enum InputMode {
    Normal,
    EnterPath,
    EnterPassword,
}

pub struct Editor {
    pub storage: MemoryBuffer,
    pub cursor_position: usize,
    pub scroll_offset: u16,
    pub last_input: Instant,
    pub idle_timeout: Option<Duration>,
    pub ttl_expiry: Option<Instant>,
    pub show_markdown: bool,
    pub read_only: bool,

    // Save functionality
    pub input_mode: InputMode,
    pub path_buffer: String,
    pub password_buffer: String,
    pub status_message: Option<(String, Instant)>, // Message and timestamp
}

impl Editor {
    pub fn new(
        idle_timeout_secs: Option<f64>,
        ttl_minutes: Option<f64>,
        encryption_key: Option<[u8; 32]>,
        read_only: bool,
    ) -> Self {
        let now = Instant::now();
        Self {
            storage: MemoryBuffer::new(1024 * 64, encryption_key), // 64KB pinned storage
            cursor_position: 0,
            scroll_offset: 0,
            last_input: now,
            idle_timeout: idle_timeout_secs.map(Duration::from_secs_f64),
            ttl_expiry: ttl_minutes.map(|m| now + Duration::from_secs_f64(m * 60.0)),
            show_markdown: false,
            read_only,
            input_mode: InputMode::Normal,
            path_buffer: String::new(),
            password_buffer: String::new(),
            status_message: None,
        }
    }

    pub fn handle_input(&mut self, ch: char) {
        match self.input_mode {
            InputMode::Normal => {
                if self.read_only {
                    return;
                }
                let mut content = self.storage.to_string();
                let byte_idx = content
                    .char_indices()
                    .map(|(i, _)| i)
                    .nth(self.cursor_position)
                    .unwrap_or(content.len());
                content.insert(byte_idx, ch);
                self.storage.update(&content);
                content.zeroize();
                self.cursor_position += 1;
            }
            InputMode::EnterPath => {
                self.path_buffer.push(ch);
            }
            InputMode::EnterPassword => {
                self.password_buffer.push(ch);
            }
        }
        self.last_input = Instant::now();
    }

    pub fn delete_backspace(&mut self) {
        match self.input_mode {
            InputMode::Normal => {
                if self.read_only {
                    return;
                }
                if self.cursor_position > 0 {
                    let mut content = self.storage.to_string();
                    self.cursor_position -= 1;
                    if let Some((byte_idx, _)) = content.char_indices().nth(self.cursor_position) {
                        content.remove(byte_idx);
                        self.storage.update(&content);
                    }
                    content.zeroize();
                }
            }
            InputMode::EnterPath => {
                self.path_buffer.pop();
            }
            InputMode::EnterPassword => {
                self.password_buffer.pop();
            }
        }
        self.last_input = Instant::now();
    }

    pub fn handle_newline(&mut self) {
        match self.input_mode {
            InputMode::Normal => {
                if !self.read_only {
                    self.handle_input('\n');
                }
            }
            InputMode::EnterPath => {
                if !self.path_buffer.trim().is_empty() {
                    self.input_mode = InputMode::EnterPassword;
                }
            }
            InputMode::EnterPassword => {
                if !self.password_buffer.is_empty() {
                    if self.password_buffer.len() < 8 {
                        self.set_status("PASSWORD TOO SHORT (MIN 8 CHARS)");
                        return;
                    }
                    // Perform Save
                    let content = self.storage.to_string();
                    let mut final_path = self.path_buffer.trim().to_string();
                    if !final_path.ends_with(".amnesio") && !final_path.contains('.') {
                        final_path.push_str(".amnesio");
                    }

                    let result =
                        persistence::save_encrypted(&final_path, &content, &self.password_buffer);

                    match result {
                        Ok(_) => {
                            self.set_status(&format!("Saved as: {}", final_path));
                        }
                        Err(e) => {
                            self.set_status(&format!("Error: {}", e));
                        }
                    }

                    // Cleanup
                    self.password_buffer.zeroize();
                    self.password_buffer.clear();
                    self.input_mode = InputMode::Normal;
                }
            }
        }
    }

    pub fn enter_save_mode(&mut self) {
        if self.read_only {
            self.set_status("Cannot save in Read-Only mode.");
            return;
        }
        self.input_mode = InputMode::EnterPath;
        self.path_buffer.clear();
        self.password_buffer.clear();
    }

    pub fn exit_popup(&mut self) {
        self.input_mode = InputMode::Normal;
        self.password_buffer.zeroize();
        self.password_buffer.clear();
        self.path_buffer.clear();
    }

    pub fn move_cursor(&mut self, offset: isize) {
        if self.input_mode != InputMode::Normal {
            return;
        }

        let mut content = self.storage.to_string();
        let char_count = content.chars().count();
        let new_pos = (self.cursor_position as isize + offset)
            .max(0)
            .min(char_count as isize);
        self.cursor_position = new_pos as usize;
        content.zeroize();
        self.last_input = Instant::now();
    }

    pub fn move_cursor_lineal(&mut self, direction: isize) {
        if self.input_mode != InputMode::Normal {
            return;
        }

        let mut content = self.storage.to_string();
        let chars: Vec<char> = content.chars().collect();
        let mut cur_line = 0;
        let mut cur_col = 0;
        let mut lines: Vec<Vec<char>> = vec![vec![]];

        for (i, c) in chars.iter().enumerate() {
            if i == self.cursor_position {
                cur_line = lines.len() - 1;
                cur_col = lines.last().unwrap().len();
            }
            if *c == '\n' {
                lines.push(vec![]);
            } else {
                lines.last_mut().unwrap().push(*c);
            }
        }

        if self.cursor_position == chars.len() {
            cur_line = lines.len() - 1;
            cur_col = lines.last().unwrap().len();
        }

        let target_line = (cur_line as isize + direction)
            .max(0)
            .min(lines.len() as isize - 1) as usize;
        let target_col = cur_col.min(lines[target_line].len());

        let mut new_idx = 0;
        for i in 0..target_line {
            new_idx += lines[i].len() + 1;
        }
        new_idx += target_col;

        self.cursor_position = new_idx;
        content.zeroize();
        self.last_input = Instant::now();
    }

    pub fn toggle_markdown(&mut self) {
        self.show_markdown = !self.show_markdown;
        self.last_input = Instant::now();
    }

    pub fn set_status(&mut self, msg: &str) {
        self.status_message = Some((msg.to_string(), Instant::now()));
    }

    pub fn is_timed_out(&self) -> bool {
        let now = Instant::now();
        if let Some(timeout) = self.idle_timeout {
            if now.duration_since(self.last_input) >= timeout {
                return true;
            }
        }
        if let Some(expiry) = self.ttl_expiry {
            if now >= expiry {
                return true;
            }
        }
        false
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        let mut content = self.storage.to_string();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(1)])
            .split(frame.area());

        let area = chunks[0];
        let height = area.height.saturating_sub(2) as usize;

        // Calculate current line and column for cursor
        let mut cur_line = 0;
        let mut cur_col = 0;
        let chars: Vec<char> = content.chars().collect();
        for i in 0..self.cursor_position {
            if i >= chars.len() {
                break;
            }
            if chars[i] == '\n' {
                cur_line += 1;
                cur_col = 0;
            } else {
                cur_col += 1;
            }
        }

        if cur_line < self.scroll_offset as usize {
            self.scroll_offset = cur_line as u16;
        } else if cur_line >= (self.scroll_offset as usize + height) {
            self.scroll_offset = (cur_line - height + 1) as u16;
        }

        let title_extra = if self.show_markdown { " [MD VIEW]" } else { "" };
        let read_only_tag = if self.read_only { " [READ-ONLY]" } else { "" };

        let editor_block = Block::default()
            .borders(Borders::ALL)
            .title(format!(
                " amnesia - volatile-only notepad{}{}",
                title_extra, read_only_tag
            ))
            .border_style(Style::default().fg(if self.read_only {
                Color::Red
            } else {
                Color::DarkGray
            }));

        let widget = if self.show_markdown {
            let lines = self.render_markdown(&content);
            Paragraph::new(lines)
        } else {
            Paragraph::new(content.clone())
                .style(Style::default().fg(Color::White).bg(Color::Black))
        };

        let widget = widget.block(editor_block).scroll((self.scroll_offset, 0));

        frame.render_widget(widget, area);

        if !self.show_markdown && self.input_mode == InputMode::Normal {
            frame.set_cursor_position((
                area.x + 1 + cur_col as u16,
                area.y + 1 + (cur_line - self.scroll_offset as usize) as u16,
            ));
        }

        // Status bar
        let stealth_tag = if self.storage.is_encrypted() {
            "[STEALTH] "
        } else {
            ""
        };

        let status_text = if let Some((msg, time)) = &self.status_message {
            if time.elapsed().as_secs() < 3 {
                format!(" {}", msg)
            } else {
                self.status_message = None;
                // Revert to default status
                format!(
                    " {}{}:{} | Idle: {}/{}s | TTL: {}",
                    stealth_tag,
                    cur_line + 1,
                    cur_col + 1,
                    self.last_input.elapsed().as_secs(),
                    self.idle_timeout
                        .map(|d| d.as_secs().to_string())
                        .unwrap_or("∞".into()),
                    self.ttl_expiry
                        .map(|e| if Instant::now() >= e {
                            0
                        } else {
                            e.duration_since(Instant::now()).as_secs()
                        })
                        .map(|s| s.to_string())
                        .unwrap_or("∞".into())
                )
            }
        } else {
            format!(
                " {}{}:{} | Idle: {}/{}s | TTL: {}",
                stealth_tag,
                cur_line + 1,
                cur_col + 1,
                self.last_input.elapsed().as_secs(),
                self.idle_timeout
                    .map(|d| d.as_secs().to_string())
                    .unwrap_or("∞".into()),
                self.ttl_expiry
                    .map(|e| {
                        if Instant::now() >= e {
                            0
                        } else {
                            e.duration_since(Instant::now()).as_secs()
                        }
                    })
                    .map(|s| s.to_string())
                    .unwrap_or("∞".into())
            )
        };

        let status_bar = Paragraph::new(status_text)
            .style(Style::default().fg(Color::Black).bg(Color::DarkGray));
        frame.render_widget(status_bar, chunks[1]);

        // Render Popup if needed
        if self.input_mode != InputMode::Normal {
            let block = Block::default()
                .title(match self.input_mode {
                    InputMode::EnterPath => " 1. Enter Filename (.amnesio) ",
                    InputMode::EnterPassword => " 2. Enter Password ",
                    _ => "",
                })
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan));

            let area = centered_rect(60, 20, frame.area());
            frame.render_widget(Clear, area); // Clear background

            let input_text = match self.input_mode {
                InputMode::EnterPath => self.path_buffer.clone(),
                InputMode::EnterPassword => "*".repeat(self.password_buffer.len()),
                _ => String::new(),
            };

            let p = Paragraph::new(input_text)
                .block(block)
                .alignment(Alignment::Center);
            frame.render_widget(p, area);
        }

        content.zeroize();
    }

    fn render_markdown<'a>(&self, content: &'a str) -> Vec<Line<'a>> {
        let mut lines = Vec::new();
        for raw_line in content.lines() {
            let mut spans = Vec::new();
            if raw_line.starts_with("# ") {
                spans.push(Span::styled(
                    raw_line,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ));
            } else if raw_line.starts_with("## ") {
                spans.push(Span::styled(
                    raw_line,
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::BOLD),
                ));
            } else if raw_line.starts_with("### ") {
                spans.push(Span::styled(
                    raw_line,
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ));
            } else {
                let parts: Vec<&str> = raw_line.split("**").collect();
                for (i, part) in parts.iter().enumerate() {
                    if i % 2 == 1 {
                        spans.push(Span::styled(
                            *part,
                            Style::default()
                                .add_modifier(Modifier::BOLD)
                                .fg(Color::Yellow),
                        ));
                    } else {
                        spans.push(Span::raw(*part));
                    }
                }
            }
            lines.push(Line::from(spans));
        }
        if content.ends_with('\n') {
            lines.push(Line::from(""));
        }
        lines
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
