use crate::mem_buffer::MemoryBuffer;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::time::{Duration, Instant};
use zeroize::Zeroize;

pub struct Editor {
    pub storage: MemoryBuffer,
    pub cursor_position: usize,
    pub scroll_offset: u16,
    pub last_input: Instant,
    pub idle_timeout: Option<Duration>,
    pub ttl_expiry: Option<Instant>,
}

impl Editor {
    pub fn new(
        idle_timeout_secs: Option<f64>,
        ttl_minutes: Option<f64>,
        encryption_key: Option<[u8; 32]>,
    ) -> Self {
        let now = Instant::now();
        Self {
            storage: MemoryBuffer::new(1024 * 64, encryption_key), // 64KB pinned storage
            cursor_position: 0,
            scroll_offset: 0,
            last_input: now,
            idle_timeout: idle_timeout_secs.map(Duration::from_secs_f64),
            ttl_expiry: ttl_minutes.map(|m| now + Duration::from_secs_f64(m * 60.0)),
        }
    }

    pub fn handle_input(&mut self, ch: char) {
        let mut content = self.storage.to_string();
        // Find the byte index for the current character position.
        let byte_idx = content
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.cursor_position)
            .unwrap_or(content.len());

        content.insert(byte_idx, ch);
        self.storage.update(&content);
        content.zeroize();
        self.cursor_position += 1;
        self.last_input = Instant::now();
    }

    pub fn delete_backspace(&mut self) {
        if self.cursor_position > 0 {
            let mut content = self.storage.to_string();
            self.cursor_position -= 1;

            // Find the byte index for the character to remove.
            if let Some((byte_idx, _)) = content.char_indices().nth(self.cursor_position) {
                content.remove(byte_idx);
                self.storage.update(&content);
            }

            content.zeroize();
            self.last_input = Instant::now();
        }
    }

    pub fn handle_newline(&mut self) {
        self.handle_input('\n');
    }

    pub fn move_cursor(&mut self, offset: isize) {
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
        let height = area.height.saturating_sub(2) as usize; // -2 for borders

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

        // Adjust scroll offset
        if cur_line < self.scroll_offset as usize {
            self.scroll_offset = cur_line as u16;
        } else if cur_line >= (self.scroll_offset as usize + height) {
            self.scroll_offset = (cur_line - height + 1) as u16;
        }

        let editor_block = Block::default()
            .borders(Borders::ALL)
            .title(" amnesia - volatile-only notepad ")
            .border_style(Style::default().fg(Color::DarkGray));

        let paragraph = Paragraph::new(content.clone())
            .block(editor_block)
            .style(Style::default().fg(Color::White).bg(Color::Black))
            .scroll((self.scroll_offset, 0));

        frame.render_widget(paragraph, area);

        // Position the cursor
        frame.set_cursor_position((
            area.x + 1 + cur_col as u16,
            area.y + 1 + (cur_line - self.scroll_offset as usize) as u16,
        ));

        // Status bar
        let stealth_tag = if self.storage.is_encrypted() {
            "[STEALTH] "
        } else {
            ""
        };
        let status = format!(
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
                    let remaining = if Instant::now() >= e {
                        0
                    } else {
                        e.duration_since(Instant::now()).as_secs()
                    };
                    format!("{}s left", remaining)
                })
                .unwrap_or("∞".into())
        );
        let status_bar =
            Paragraph::new(status).style(Style::default().fg(Color::Black).bg(Color::DarkGray));
        frame.render_widget(status_bar, chunks[1]);

        content.zeroize();
    }
}
