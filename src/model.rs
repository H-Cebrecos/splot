/// Abstract model the parser extracts from a source file and the renderer consumes
#[derive(Default)]
pub struct Model {
    pub clock: Clock,
    pub regions: Vec<Region>,
    pub signals: Vec<Signal>,
    pub notes: Vec<Note>,
}

impl Model {
    pub fn parse(input: &str) -> Self {
        crate::parser::parse(input)
    }

    fn cycles(&self) -> usize {
        //compute based on the max of regions and signals
        todo!()
    }

    /// Width of the widest row label that will be drawn. The clock only
    /// counts when it is visible.
    fn label_width(&self) -> usize {
        let clk = if self.clock.visible { "clk".len() } else { 0 };
        self.signals
            .iter()
            .map(|s| s.name.chars().count()) // chars, since quoted names may be non-ASCII
            .fold(clk, usize::max)
    }

    /// Column where every row's waveform starts: the labels plus one space.
    /// With no labels there is no gutter at all.
    pub fn gutter(&self) -> usize {
        match self.label_width() {
            0 => 0,
            w => w + 1,
        }
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
    pub fn columns_per_phase(&self) -> usize {
        self.width as usize + 1
    }

    /// Whether the edge that starts `phase` is a rising one.
    pub fn is_rising(&self, phase: usize) -> bool {
        (phase % 2 == 0) == (self.edge == ClockEdge::Rising)
    }
}

pub struct Region {
    /// `None` for `_`: boundaries are drawn but no label.
    pub label: Option<String>,
    /// Length in cycles; `None` means "until the end of the diagram".
    pub duration: Option<usize>,
    /// Region end can stretch (`···`) at its end.
    pub stretch: bool,
}

struct Signal {
    name: String,
}

struct Marker;

pub struct Note {
    pub id: String,
    pub text: String,
}
