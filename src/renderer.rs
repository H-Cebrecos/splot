// each renderer take a start row and return the number of rows it used, with the caller keeping the cursor

use core::fmt;

use crate::model::*;

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
        cursor = render_regions(cursor, &model, &mut canvas);
        cursor = render_clk(cursor, &model, &mut canvas);
        cursor = render_signals(cursor, &model, &mut canvas);
        cursor = render_notes(cursor + 1, &model.notes, &mut canvas);
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

    /// True if every cell of `cols` on `row` is still empty (unwritten or a space).
    fn is_free(&self, row: usize, cols: std::ops::Range<usize>) -> bool {
        cols.into_iter().all(|c| {
            self.rows
                .get(row)
                .and_then(|line| line.get(c))
                .map_or(true, |&ch| ch == ' ')
        })
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

/// Returns which cycles have stretching
fn stretched_cycles(model: &Model) -> Vec<usize> {
    let mut stretch = Vec::<usize>::new();
    let mut start = 0;
    for reg in &model.regions {
        if let Some(dur) = reg.duration {
            start += dur;
            if reg.stretch {
                stretch.push(start - 1);
            }
        }
    }
    stretch
}

/// Columns of a stretched cycle that hold the break marks: the two centre
/// columns of the cycle. It needs a plateau column next to the middle edge,
/// so a clock with width 0 never shows a break.
fn is_stretch_col(offset: usize, phase_cols: usize) -> bool {
    phase_cols >= 2 && (offset == phase_cols - 1 || offset == phase_cols)
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
fn render_clk(cursor: usize, model: &Model, canvas: &mut Canvas) -> usize {
    let gutter = model.gutter();
    let cycles = model.cycles();
    let clock = &model.clock;
    let mut cursor = cursor;

    let stretch = stretched_cycles(model);

    if !clock.visible {
        return cursor;
    }

    if clock.num {
        cursor = cursor + 1;
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

    // Stretched cycles: dots on both rows, replacing the middle edge and its neighbours.
    let cycle_cols = 2 * phase_cols;
    for &cycle in &stretch {
        for offset in (0..cycle_cols).filter(|&o| is_stretch_col(o, phase_cols)) {
            let col = gutter + cycle * cycle_cols + offset;
            canvas.put(cursor, col, '⌠');
            canvas.put(cursor + 1, col, '⌡');
        }
    }

    if clock.num {
        for cycle in 0..=cycles {
            let col = gutter + cycle * cycle_cols;
            // A label wider than one column continues into the columns to its right
            for (offset, digit) in cycle.to_string().chars().enumerate() {
                canvas.put(cursor - 1, col + offset, digit);
            }
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
fn render_regions(cursor: usize, model: &Model, canvas: &mut Canvas) -> usize {
    let gutter = model.gutter();
    let total_cycles = model.cycles();
    let regions = &model.regions;
    let clock = &model.clock;

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
            draw_label(canvas, cursor, left, right, name);
        }

        start = end;
    }

    canvas.put(cursor, gutter + start * cycle_cols, '┤');
    cursor + 1
}

/// Draws `╴text╶` centered between the boundary columns `left` and `right`,
/// or a `*` when it doesn't fit.
fn draw_label(canvas: &mut Canvas, row: usize, left: usize, right: usize, text: &str) {
    let room = right - left - 1;
    if room == 0 {
        return;
    }
    let boxed = format!("╴{text}╶");
    let label = if boxed.chars().count() <= room {
        boxed
    } else {
        "*".to_string()
    };
    // Integer division puts the odd leftover column after the label.
    let offset = (room - label.chars().count()) / 2;
    canvas.text(row, left + 1 + offset, &label);
}

/// The glyph filling a segment's columns.
fn level_glyph(value: &Value) -> char {
    match value {
        Value::Low => '▁',
        Value::High => '▔',
        Value::Unknown => '░',
        Value::Bus(_) => '─',
    }
}

/// The glyph at a segment boundary, joining the previous value to the next one.
fn boundary_glyph(previous: Option<&Value>, next: &Value) -> char {
    match (previous, next) {
        // The first segment has no leading transition.
        (None, next) => level_glyph(next),
        (Some(Value::Low), Value::High) => '╱',
        (Some(Value::High), Value::Low) => '╲',
        // Equal levels continue straight. Two buses never do, even with the
        // same text, so they stay visibly separate.
        (Some(prev), next) if prev == next && !matches!(next, Value::Bus(_)) => level_glyph(next),
        // Anything involving unknown or a bus.
        _ => '╳',
    }
}

/// Draws one row per signal. A signal ends where its last segment ends.
/// Returns the next free row.
fn render_signals(cursor: usize, model: &Model, canvas: &mut Canvas) -> usize {
    let gutter = model.gutter();
    let total_cycles = model.cycles();
    let clock = &model.clock;
    let signals = &model.signals;
    let regions = &model.regions;

    let stretch = stretched_cycles(model);

    let cycle_cols = 2 * clock.columns_per_phase();
    let mut row = cursor;
    // The previous signal asked for a delimiter row below it.
    let mut bot_pending = false;

    for (i, signal) in signals.iter().enumerate() {
        let delim = bot_pending || signal.top_delim;
        if delim {
            render_delimiters(row, regions, total_cycles, clock, gutter, canvas);
        }
        // Annotations start on the delimiter row. A gap with only a delimiter
        // still takes one row, and one with neither takes none.
        row = render_annotations(row, i, model, canvas).max(row + delim as usize);

        canvas.text(row, 0, &signal.name);

        let mut start = 0;
        let mut end = 0;
        let mut previous: Option<&Value> = None;

        for segment in &signal.segments {
            end = start + segment.cycles;
            let left = gutter + start * cycle_cols;
            let right = gutter + end * cycle_cols;

            for col in left + 1..right {
                canvas.put(row, col, level_glyph(&segment.value));
            }
            canvas.put(row, left, boundary_glyph(previous, &segment.value));

            if let Value::Bus(text) = &segment.value {
                draw_label(canvas, row, left, right, text);
            }

            previous = Some(&segment.value);
            start = end;
        }
        // Stretched cycles show three dots whatever the value is, bus labels included.
        // Only cycles this signal covers, and only when a phase has two columns or more.
        let phase_cols = cycle_cols / 2;
        if phase_cols >= 2 {
            for &cycle in stretch.iter().filter(|&&c| c < end) {
                // The middle of the cycle and one column on each side.
                let middle = gutter + cycle * cycle_cols + phase_cols;
                canvas.text(row, middle - 1, "···");
            }
        }

        // Signals before and after the diagram are unknown, draw trasition at the boundaries
        canvas.put(row, gutter, '╳');
        canvas.put(row, gutter + end * cycle_cols, '╳');

        // Draw clock tail
        if clock.tail {
            canvas.text(row, gutter + cycle_cols * total_cycles + 1, "···");
        }

        row += 1;
        bot_pending = signal.bot_delim;
    }

    // The gap after the last signal.
    if bot_pending {
        render_delimiters(row, regions, total_cycles, clock, gutter, canvas);
    }
    row = render_annotations(row, signals.len(), model, canvas).max(row + bot_pending as usize);

    row
}

fn render_delimiters(
    cursor: usize,
    regions: &[Region],
    total_cycles: usize,
    clock: &Clock,
    gutter: usize,
    canvas: &mut Canvas,
) {
    if regions.is_empty() {
        return;
    }

    let cycle_cols = 2 * clock.columns_per_phase();
    let mut start = 0;

    for region in regions.iter() {
        let end = match region.duration {
            Some(duration) => start + duration,
            None => total_cycles.max(start),
        };
        let pos = gutter + start * cycle_cols;

        canvas.put(cursor, pos, '╎');

        start = end;
    }

    canvas.put(cursor, gutter + start * cycle_cols, '╎');
}

/// Draws every annotation sitting in the gap before signal `index`, one row each.
/// Returns the next free row.
fn render_annotations(row: usize, index: usize, model: &Model, canvas: &mut Canvas) -> usize {
    let gutter = model.gutter();
    let cycle_cols = 2 * model.clock.columns_per_phase();
    // Column of the closing edge: labels must not cross it.
    let end = gutter + model.cycles() * cycle_cols;
    let mut any = false;

    for annotation in model.annotations.iter().filter(|a| a.row == index) {
        // Corner for a label growing to the right, and for one growing to the left.
        let (corner_right, corner_left) = match annotation.pos {
            Position::Above => ('╭', '╮'),
            Position::Below => ('╰', '╯'),
        };

        for marker in &annotation.marks {
            let col = gutter + marker.cycle * cycle_cols;
            let width = marker.text.chars().count() + 1; // text plus `╴` or `╶`

            let right = col + 1..col + 1 + width;
            let left = col.saturating_sub(width)..col;

            // Right unless it would cross the end of the diagram or hit something
            // already drawn, and only if the left side is actually free.
            let right_blocked = right.end > end || !canvas.is_free(row, right);
            let go_left =
                right_blocked && left.start >= gutter && canvas.is_free(row, left.clone());

            if go_left {
                canvas.text(row, left.start, &format!("{}╶", marker.text));
                canvas.put(row, col, corner_left);
            } else {
                canvas.put(row, col, corner_right);
                canvas.text(row, col + 1, &format!("╴{}", marker.text));
            }
        }
        any = true;
    }

    // Every annotation in the gap shares one row.
    row + any as usize
}
