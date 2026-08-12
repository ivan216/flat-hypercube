use crate::commands::{CommandOutput, OUTPUT_PANEL_MAX_HEIGHT};
use crossterm::{
    QueueableCommand, cursor,
    style::{self, Stylize},
    terminal,
};
use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenLayout {
    pub puzzle_height: u16,
    pub output_top: u16,
    pub output_height: u16,
    pub rev_row: u16,
    pub status_row: u16,
}

impl ScreenLayout {
    pub fn new(term_h: u16, output_visible: bool) -> Self {
        let base_rows = term_h.min(2);
        let output_height = if output_visible {
            term_h
                .saturating_sub(base_rows + 1)
                .min(OUTPUT_PANEL_MAX_HEIGHT)
        } else {
            0
        };
        let reserved = output_height + base_rows;
        let puzzle_height = term_h.saturating_sub(reserved).max(1);
        let output_top = puzzle_height;
        let rev_row = term_h.saturating_sub(2);
        let status_row = term_h.saturating_sub(1);

        Self {
            puzzle_height,
            output_top,
            output_height,
            rev_row,
            status_row,
        }
    }
}

pub fn draw_output_panel(
    stdout: &mut io::Stdout,
    output: &CommandOutput,
    layout: ScreenLayout,
    term_w: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    if layout.output_height == 0 {
        return Ok(());
    }

    let end = output.lines.len().saturating_sub(output.scroll);
    let start = end.saturating_sub(layout.output_height as usize);
    let visible = &output.lines[start..end];

    for i in 0..layout.output_height {
        stdout
            .queue(cursor::MoveTo(0, layout.output_top + i))?
            .queue(terminal::Clear(terminal::ClearType::CurrentLine))?;
        if let Some(line) = visible.get(i as usize) {
            stdout.queue(style::Print(truncate_to_width(line, term_w)))?;
        }
    }

    Ok(())
}

pub fn draw_status_line(
    stdout: &mut io::Stdout,
    output: &CommandOutput,
    layout: ScreenLayout,
    term_w: u16,
    message: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    stdout
        .queue(cursor::MoveTo(0, layout.status_row))?
        .queue(terminal::Clear(terminal::ClearType::CurrentLine))?;

    let show_hint = output.hint.as_deref() == Some(message) && !output.open;
    if !show_hint {
        stdout.queue(style::Print(truncate_to_width(message, term_w)))?;
        return Ok(());
    }

    const OUTPUT_PANEL_HINT: &str = " (press ? to toggle panel)";
    let hint_len = OUTPUT_PANEL_HINT.chars().count() as u16;
    let message_width = term_w.saturating_sub(hint_len);
    let display_message = truncate_to_width(message, message_width.max(1));
    let used = display_message.chars().count() as u16;
    stdout.queue(style::Print(display_message))?;

    if used < term_w {
        let hint = truncate_to_width(OUTPUT_PANEL_HINT, term_w - used);
        stdout.queue(style::PrintStyledContent(hint.with(style::Color::DarkGrey)))?;
    }

    Ok(())
}

fn truncate_to_width(line: &str, width: u16) -> String {
    line.chars().take(width as usize).collect()
}
