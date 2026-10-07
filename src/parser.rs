use crate::model::*;

pub fn parse(src: &str) -> Model {
    let mut model = Model::default();

    for line in src.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // The keyword is the first word, whether or not a space precedes the
        // colon (`clk: 2` and `clk : 2`). A quoted name keeps its quotes, so it
        // never equals a keyword and falls through to the signal case.
        let keyword = line
            .split(|c: char| c == ':' || c.is_whitespace())
            .next()
            .unwrap_or("");

        match keyword {
            "clk" => model.clock = parse_clk(line),
            // Last line wins, so a new regions line replaces the previous one.
            "regions" => model.regions = parse_regions(line),
            "note" => {
                if let Some(note) = parse_note(line) {
                    model.notes.push(note);
                }
            }
            // Markers are not parsed yet.
            "^" | "v" => todo!(),
            // Remaining stuff with a semicolon are signals (if valid)
            _ if line.contains(':') => todo!(), //model.signals.push(Signal),
            // Ignore unknown lines
            _ => {}
        }
    }

    model
}

fn parse_clk(line: &str) -> Clock {
    let Some((_, args)) = line.split_once(':') else {
        return Clock::default();
    };

    let mut toks = args.split_whitespace().peekable();
    let style = match toks.peek() {
        Some(&"square") => {
            toks.next();
            ClockStyle::Square
        }
        Some(&"analog") => {
            toks.next();
            ClockStyle::Analog
        }
        Some(&"rounded") => {
            toks.next();
            ClockStyle::Rounded
        }
        _ => ClockStyle::Square,
    };

    let edge = match toks.peek() {
        Some(&"rising") => {
            toks.next();
            ClockEdge::Rising
        }
        Some(&"falling") => {
            toks.next();
            ClockEdge::Falling
        }
        _ => ClockEdge::Rising,
    };

    let width: u8 = match toks.peek() {
        Some(x) => {
            if let Ok(x) = x.parse() {
                toks.next();
                x
            } else {
                0
            }
        }
        None => 0,
    };

    let tail = match toks.next() {
        Some("...") => true,
        _ => false,
    };

    //NOTE: we tolerate extra tokens in this line, we already have all info parsed.

    Clock {
        visible: true,
        style,
        edge,
        width,
        tail,
    }
}

fn parse_note(line: &str) -> Option<Note> {
    let line = line.trim_start_matches("note");
    let Some((name, body)) = line.split_once(':') else {
        return None;
    };
    Some(Note {
        id: name.to_string(),
        text: body.to_string(),
    })
}

/// Parses the `regions:` line. Pieces that are empty are skipped, which is
/// what a half-typed line looks like.
fn parse_regions(line: &str) -> Vec<Region> {
    let body = line.split_once(':').map_or("", |(_, rest)| rest);
    body.split('|').filter_map(parse_region).collect()
}

fn parse_region(piece: &str) -> Option<Region> {
    let piece = piece.trim();
    if piece.is_empty() {
        return None;
    }

    // A piece without a colon is still a region, just with no duration.
    let (name, spec) = piece.split_once(':').unwrap_or((piece, ""));
    let name = name.trim();
    let spec = spec.trim();

    Some(Region {
        label: (!name.is_empty() && name != "_").then(|| name.to_string()),
        // A malformed number simply becomes "until the end".
        duration: spec.trim_end_matches('+').trim().parse().ok(),
        stretch: spec.ends_with('+'),
    })
}
