// each renderer take a start row and return the number of rows it used, with the caller keeping the cursor

use core::fmt;

use crate::model::{Clock, ClockStyle, Model, Note, Region};

/// A grid of characters that grows on write. Writing a space leaves
/// whatever was already there, so later draws overlay earlier ones.
#[derive(Default)]
pub struct Canvas {
    rows: Vec<Vec<char>>,
}

impl Canvas {
    /// Renders a model as a `Canvas`
    pub fn render(model: &Model) -> Self {
        let mut canvas = Self::default();
        let mut cursor = 0;
        cursor = render_regions(
            cursor,
            &model.regions,
            10,
            &model.clock,
            model.gutter(),
            &mut canvas,
        );
        cursor = render_clk(cursor, &model.clock, 10, model.gutter(), &mut canvas);
        cursor = render_notes(cursor, &model.notes, &mut canvas);
        _ = cursor;
        canvas
    }

    /// Writes `ch` at (row, col), growing the canvas as needed.
    fn put(&mut self, row: usize, col: usize, ch: char) {
        // spaces are transparent
        if ch == ' ' {
            return;
        }
        if self.rows.len() <= row {
            self.rows.resize_with(row + 1, Vec::new);
        }
        let line = &mut self.rows[row];
        if line.len() <= col {
            line.resize(col + 1, ' ');
        }
        line[col] = ch;
    }

    /// Writes `s` starting at (row, col), one char per column.
    fn text(&mut self, row: usize, col: usize, s: &str) {
        for (i, ch) in s.chars().enumerate() {
            self.put(row, col + i, ch);
        }
    }

    /// Width of the widest row. Rows only ever end on a drawn glyph, so this
    /// is the column just past the rightmost thing on the canvas.
    fn width(&self) -> usize {
        self.rows.iter().map(Vec::len).max().unwrap_or(0)
    }
}

// One line per row, trailing spaces trimmed.
impl fmt::Display for Canvas {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in &self.rows {
            let line: String = row.iter().collect();
            writeln!(f, "{}", line.trim_end())?;
        }
        Ok(())
    }
}

/// Glyphs LUT for the top and bottom row of an edge column. An edge is one
/// vertical stroke joining the high level and the low level.
fn edge_glyphs(style: ClockStyle, rising: bool) -> (char, char) {
    match (style, rising) {
        (ClockStyle::Square, true) => ('┌', '┘'),
        (ClockStyle::Square, false) => ('┐', '└'),
        (ClockStyle::Analog, true) => ('╭', '┘'),
        (ClockStyle::Analog, false) => ('┐', '╰'),
        (ClockStyle::Rounded, true) => ('╭', '╯'),
        (ClockStyle::Rounded, false) => ('╮', '╰'),
    }
}

/// Draws the clock on two rows starting at `cursor`, for `cycles` cycles.
/// Returns the next free row.
pub fn render_clk(
    cursor: usize,
    clock: &Clock,
    cycles: usize,
    gutter: usize,
    canvas: &mut Canvas,
) -> usize {
    if !clock.visible {
        return cursor;
    }

    // Draw the label in the line where the clock starts
    let label_row = if clock.is_rising(0) {
        cursor + 1
    } else {
        cursor
    };
    canvas.text(label_row, 0, "clk");

    let phase_cols = clock.columns_per_phase();
    let last = 2 * cycles;

    for phase in 0..=last {
        let col = gutter + phase * phase_cols;
        let rising = clock.is_rising(phase);

        let (top, bottom) = edge_glyphs(clock.style, rising);
        canvas.put(cursor, col, top);
        canvas.put(cursor + 1, col, bottom);

        // The last edge closes the diagram, so nothing follows it.
        if phase == last {
            break;
        }

        // After an edge determine the line (top/bottom) to continue the signal.
        let level_row = if rising { cursor } else { cursor + 1 };
        for i in 1..phase_cols {
            canvas.put(level_row, col + i, '─');
        }
    }

    if clock.tail && cycles > 0 {
        let level_row = if clock.is_rising(2 * cycles) {
            cursor
        } else {
            cursor + 1
        };
        canvas.text(level_row, gutter + 2 * cycles * phase_cols + 1, "···");
    }

    cursor + 2
}

/// Splits `text` into lines of at most `width` chars, breaking at spaces.
/// A word longer than `width` gets a line of its own and overflows it.
fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for word in text.split_whitespace() {
        match lines.last_mut() {
            Some(line) if line.chars().count() + 1 + word.chars().count() <= width => {
                line.push(' ');
                line.push_str(word);
            }
            _ => lines.push(word.to_string()),
        }
    }
    lines
}

/// Draws each note as `id: text`, wrapped to the current canvas width.
/// Returns the next free row.
fn render_notes(cursor: usize, notes: &[Note], canvas: &mut Canvas) -> usize {
    // Measured once up front so a long note can't widen the limit for the
    // ones after it.
    let width = canvas.width();
    let mut row = cursor;

    for note in notes {
        let prefix = format!("{}: ", note.id);
        let indent = prefix.chars().count();

        // Continuation lines hang under the first word of the text.
        let mut lines = wrap(&note.text, width.saturating_sub(indent));
        if lines.is_empty() {
            // A note with no text still shows its id.
            lines.push(String::new());
        }

        canvas.text(row, 0, &prefix);
        for line in &lines {
            canvas.text(row, indent, line);
            row += 1;
        }
    }

    row
}

/// Draws the region boundaries on one row at `cursor`. Returns the next free row.
fn render_regions(
    cursor: usize,
    regions: &[Region],
    total_cycles: usize,
    clock: &Clock,
    gutter: usize,
    canvas: &mut Canvas,
) -> usize {
    if regions.is_empty() {
        return cursor;
    }

    let cycle_cols = 2 * clock.columns_per_phase();
    let mut start = 0;

    for (i, region) in regions.iter().enumerate() {
        let end = match region.duration {
            Some(duration) => start + duration,
            None => total_cycles.max(start),
        };
        let left = gutter + start * cycle_cols;
        let right = gutter + end * cycle_cols;

        for col in left + 1..right {
            canvas.put(cursor, col, '─');
        }
        canvas.put(cursor, left, if i == 0 { '├' } else { '┼' });

        if let Some(name) = &region.label {
            let room = right - left - 1;
            let boxed = format!("╴{name}╶");
            // `*` stands in for any label that doesn't fit.
            let text = if boxed.chars().count() <= room {
                boxed
            } else {
                "*".to_string()
            };
            if room > 0 {
                // Integer division puts the odd leftover dash after the label.
                let offset = (room - text.chars().count()) / 2;
                canvas.text(cursor, left + 1 + offset, &text);
            }
        }

        start = end;
    }

    canvas.put(cursor, gutter + start * cycle_cols, '┤');
    cursor + 1
}
