use crate::{
    helpers::{
        build_target_lines_from_layout, build_typed_visible_from_layout,
        cursor_row_col_from_layout, generate_text, layout_text,
    },
    types::TextSource,
};

use ratatui::{
    crossterm::event::{self, KeyCode},
    prelude::*,
    widgets::*,
};
use std::time::Instant;
use tui_input::{Input, InputRequest};

pub struct App {
    source: TextSource,
    target: String,
    input: Input,
    started_at: Option<Instant>,
    finished_at: Option<Instant>,
    count: usize,
}

impl App {
    pub fn new(source: TextSource, count: usize) -> Self {
        let target = match &source {
            TextSource::RandomWords(dict) => generate_text(dict, count),
            TextSource::Fixed(text) => text.clone(),
        };

        Self {
            source,
            target,
            input: Input::default(),
            started_at: None,
            finished_at: None,
            count,
        }
    }

    pub fn reset(&mut self) {
        self.target = match &self.source {
            TextSource::RandomWords(dict) => generate_text(dict, self.count),
            TextSource::Fixed(text) => text.clone(),
        };
        self.input = Input::default();
        self.started_at = None;
        self.finished_at = None;
    }

    pub fn handle_key(&mut self, key: event::KeyEvent) {
        if self.finished_at.is_some() {
            match key.code {
                KeyCode::Enter => self.reset(),
                KeyCode::Esc => {}
                _ => {}
            }

            return;
        }

        if self.started_at.is_none() {
            self.started_at = Some(Instant::now());
        }

        match key.code {
            KeyCode::Char(c) => {
                self.input.handle(InputRequest::InsertChar(c));
            }
            KeyCode::F(5) => {
                self.reset();
            }
            KeyCode::Backspace => {
                self.input.handle(InputRequest::DeletePrevChar);
            }
            _ => {}
        }

        let typed = self.input.value();
        if typed.len() >= self.target.len() {
            self.finished_at = Some(Instant::now());
        }
    }

    pub fn stats(&self) -> (f64, f64, f64) {
        let typed = self.input.value();
        let total_typed = typed.chars().count() as u32;

        let correct = self
            .target
            .chars()
            .zip(typed.chars())
            .filter(|(a, b)| a == b)
            .count() as u32;

        let elapsed = self
            .started_at
            .map(|t| {
                if self.finished_at.is_some() {
                    self.finished_at.unwrap().duration_since(t).as_secs_f64()
                } else {
                    t.elapsed().as_secs_f64()
                }
            })
            .unwrap_or(0.0);

        let wpm = if elapsed > 0.0 {
            let minutes = elapsed / 60.0;
            if minutes > 0.0 {
                (total_typed as f64 / 5.0) / minutes
            } else {
                0.0
            }
        } else {
            0.0
        };

        let accuracy = if total_typed > 0 {
            (correct as f64 / total_typed as f64) * 100.0
        } else {
            100.0
        };

        (elapsed, wpm, accuracy)
    }

    pub fn draw_ui(&self, f: &mut Frame) {
        let area = f.area();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Length(3), // Title
                    Constraint::Min(5),    // Target (multi-line)
                    Constraint::Length(3), // Typed
                    Constraint::Length(3), // Stats
                    Constraint::Min(0),
                ]
                .as_ref(),
            )
            .split(area);

        let title = Paragraph::new("Terminal Typing").alignment(Alignment::Center);
        f.render_widget(title, chunks[0]);

        let typed_block = Block::default().title("Typed Words").borders(Borders::ALL);
        let typed_inner = typed_block.inner(chunks[2]);
        let typed_width = typed_inner.width.max(1);

        let typed_layout = layout_text(self.input.value(), typed_width);

        let (cursor_row, cursor_col) =
            cursor_row_col_from_layout(&typed_layout, self.input.cursor());

        let typed_visible_height = typed_inner.height.max(1);
        let total_lines = typed_layout.len() as u16;
        let max_scroll = total_lines.saturating_sub(typed_visible_height);
        let desired_scroll = cursor_row.saturating_sub(typed_visible_height - 1);
        let scroll_y = desired_scroll.min(max_scroll);

        let target_block = Block::default().title("Target Text").borders(Borders::ALL);
        let target_inner = target_block.inner(chunks[1]);
        let target_width = target_inner.width.max(1);
        let target_visible_height = target_inner.height.max(1);

        let target_layout = layout_text(&self.target, target_width);

        let target_lines = build_target_lines_from_layout(
            &target_layout,
            self.input.value(),
            scroll_y,
            target_visible_height,
        );

        let target_paragraph = Paragraph::new(target_lines)
            .block(target_block)
            .wrap(Wrap { trim: false });
        f.render_widget(target_paragraph, chunks[1]);

        let typed_visible =
            build_typed_visible_from_layout(&typed_layout, scroll_y, typed_visible_height);

        let typed_paragraph = Paragraph::new(typed_visible)
            .block(typed_block)
            .wrap(Wrap { trim: false });
        f.render_widget(typed_paragraph, chunks[2]);

        let cursor_screen_x = typed_inner.x + cursor_col;
        let cursor_screen_y = typed_inner.y + cursor_row.saturating_sub(scroll_y);
        f.set_cursor_position((cursor_screen_x, cursor_screen_y));

        let (elapsed, wpm, accuracy) = self.stats();
        let stats_text = format!(
            "Time: {:.1}s | WPM: {:.1} | Accuracy: {:.1}%",
            elapsed, wpm, accuracy
        );

        let status = if self.finished_at.is_some() {
            format!(
                "{} | Finished! Press Enter to start a new test or ESC to quit.",
                stats_text
            )
        } else {
            stats_text
        };

        let stats_block = Block::default().title("Stats").borders(Borders::ALL);
        let stats_paragraph = Paragraph::new(status).block(stats_block);
        f.render_widget(stats_paragraph, chunks[3]);
    }
}
