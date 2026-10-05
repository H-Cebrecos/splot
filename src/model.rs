/// Abstract model the parser extracts from a source file and the renderer consumes
#[derive(Default)]
struct Model {
    clock: Clock,
    regions: Vec<Region>,
    signals: Vec<Signal>,
    notes: Vec<Note>,
}

impl Model {
    fn cycles(&self) -> usize {
        //compute based on the max of regions and signals
        todo!()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum ClockStyle {
    #[default]
    Square,
    Analog,
    Rounded,
}

/// Which transition the waveform starts with.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum ClockEdge {
    #[default]
    Rising,
    Falling,
}

/// The time reference for the whole diagram. It always exists; `visible`
/// only controls whether a clock row is drawn. The width scales the time
/// axis even when the clock is hidden.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Clock {
    pub visible: bool,
    pub style: ClockStyle,
    pub edge: ClockEdge,
    /// Extra columns per phase beyond the edge itself (0..=255).
    pub width: u8,
    /// Draw a trailing `···` to show the waveform continues.
    pub tail: bool,
}

impl Clock {
    fn columns_per_cycle(&self) -> usize {
        2 * (self.width as usize + 1)
    }
}

struct Region;
struct Signal;
struct Marker;

struct Note {
    id: String,
    text: String,
}
