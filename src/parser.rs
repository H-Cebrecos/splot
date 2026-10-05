use crate::model::{Clock, ClockEdge, ClockStyle};

pub fn parse_clk(line: &str) -> Clock {
    if let Some((_, args)) = line.split_once(':') {
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
    } else {
        Clock::default()
    }
}
