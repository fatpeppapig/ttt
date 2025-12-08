use crate::types::{Glyph, Layout, TextSource};

use rand::Rng;
use ratatui::prelude::*;
use std::{env, fs, process};

pub fn print_usage_and_exit() -> ! {
    eprintln!(
        "Usage: ttt [-count COUNT] [-dict PATH] [-text PATH]

Options:
  -count COUNT Generate text using COUNT number of words
  -text PATH   Use practice text from file at PATH
  -dict PATH   Use dictionary file at PATH to generate a random practice text
By default, a random practice text using system dictionary is generated."
    );

    process::exit(1);
}

pub fn parse_args() -> (usize, TextSource) {
    let mut dict_path: Option<String> = None;
    let mut text_path: Option<String> = None;
    let mut count: usize = 0;

    let mut args = env::args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => print_usage_and_exit(),

            "-d" | "-dict" | "--dict" => {
                let path = args.next().unwrap_or_else(|| {
                    eprintln!("Missing path after {}", arg);

                    print_usage_and_exit()
                });

                dict_path = Some(path);
            }

            "-t" | "-text" | "--text" => {
                let path = args.next().unwrap_or_else(|| {
                    eprintln!("Missing path after {}", arg);

                    print_usage_and_exit()
                });

                text_path = Some(path);
            }

            "-c" | "-count" | "--count" => {
                count = args
                    .next()
                    .unwrap_or_else(|| {
                        eprintln!("Missing count after {}", arg);

                        print_usage_and_exit()
                    })
                    .parse::<usize>()
                    .unwrap();
            }

            other => {
                eprintln!("Unknown argument: {}", other);

                print_usage_and_exit()
            }
        }
    }

    if let Some(path) = text_path {
        let content = fs::read_to_string(&path).unwrap_or_else(|e| {
            eprintln!("Failed to read text file at {}: {}", path, e);

            process::exit(1);
        });

        let content = content.replace("\r\n", "\n");

        return (count, TextSource::Fixed(content));
    }

    let dict = if let Some(path) = dict_path {
        load_dictionary_from_file(&path)
    } else {
        load_system_dictionary()
    };

    (count, TextSource::RandomWords(dict))
}

pub fn load_dictionary_from_file(path: &str) -> Vec<String> {
    let content = fs::read_to_string(&path).unwrap_or_else(|e| {
        eprintln!("Failed to read dictionary file at {}: {}", path, e);

        process::exit(1);
    });

    content
        .lines()
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect()
}

pub fn load_system_dictionary() -> Vec<String> {
    load_dictionary_from_file("/usr/share/dict/words")
}

pub fn generate_text(dictionary: &[String], count: usize) -> String {
    let mut rng = rand::rng();

    (0..count)
        .map(|_| {
            let index = rng.random_range(0..dictionary.len());
            dictionary[index].clone()
        })
        .collect::<Vec<String>>()
        .join(" ")
}

pub fn layout_text(text: &str, width: u16) -> Layout {
    let width = width.max(1) as usize;
    let chars: Vec<char> = text.chars().collect();

    let mut lines: Layout = vec![Vec::new()];
    let mut col = 0usize;
    let mut i = 0usize;

    while i < chars.len() {
        if chars[i] == ' ' {
            if col == 0 {
                i += 1;
                continue;
            }

            if col + 1 > width {
                lines.push(Vec::new());
                col = 0;
                i += 1;

                continue;
            }

            lines.last_mut().unwrap().push(Glyph { ch: ' ', idx: i });
            col += 1;
            i += 1;

            continue;
        }

        let start = i;
        while i < chars.len() && chars[i] != ' ' {
            i += 1;
        }

        let word_len = i - start;
        if col > 0 && col + word_len > width {
            lines.push(Vec::new());
            col = 0;
        }

        for j in start..i {
            lines.last_mut().unwrap().push(Glyph {
                ch: chars[j],
                idx: j,
            });

            col += 1;
        }
    }

    if lines.is_empty() {
        lines.push(Vec::new());
    }

    lines
}

pub fn cursor_row_col_from_layout(layout: &Layout, cursor_idx: usize) -> (u16, u16) {
    for (row, line) in layout.iter().enumerate() {
        for (col, glyph) in line.iter().enumerate() {
            if glyph.idx == cursor_idx {
                return (row as u16, col as u16);
            }
        }
    }

    if let Some(last_line) = layout.last() {
        (
            layout.len().saturating_sub(1) as u16,
            last_line.len() as u16,
        )
    } else {
        (0, 0)
    }
}

pub fn build_target_lines_from_layout(
    layout: &Layout,
    typed: &str,
    scroll_y: u16,
    visible_height: u16,
) -> Vec<Line<'static>> {
    let typed_chars: Vec<char> = typed.chars().collect();

    let start = scroll_y as usize;
    let end = (scroll_y + visible_height).min(layout.len() as u16) as usize;

    let mut lines_out: Vec<Line<'static>> = Vec::new();

    for row in start..end {
        let mut spans: Vec<Span<'static>> = Vec::new();

        for glyph in &layout[row] {
            let ch = glyph.ch;
            let idx = glyph.idx;

            let style = if let Some(uc) = typed_chars.get(idx) {
                if *uc == ch {
                    Style::default().fg(Color::Green)
                } else {
                    if ch == ' ' {
                        Style::default().bg(Color::Red)
                    } else {
                        Style::default().fg(Color::Red)
                    }
                }
            } else {
                Style::default()
            };

            spans.push(Span::styled(ch.to_string(), style));
        }
        lines_out.push(Line::from(spans));
    }

    lines_out
}

pub fn build_typed_visible_from_layout(
    layout: &Layout,
    scroll_y: u16,
    visible_height: u16,
) -> String {
    let start = scroll_y as usize;
    let end = (scroll_y + visible_height).min(layout.len() as u16) as usize;

    let mut lines: Vec<String> = Vec::new();
    for row in start..end {
        let s: String = layout[row].iter().map(|g| g.ch).collect();
        lines.push(s);
    }

    lines.join("\n")
}
