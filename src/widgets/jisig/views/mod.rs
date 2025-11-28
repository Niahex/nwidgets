pub mod dayview;
pub mod weekview;
pub mod monthview;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewMode {
    Day,
    Week,
    Month,
}

impl ViewMode {
    pub fn next(&self) -> Self {
        match self {
            ViewMode::Day => ViewMode::Week,
            ViewMode::Week => ViewMode::Month,
            ViewMode::Month => ViewMode::Day,
        }
    }

    pub fn get_window_size(&self) -> (i32, i32) {
        match self {
            ViewMode::Day => (500, 600),
            ViewMode::Week => (900, 700),
            ViewMode::Month => (1100, 800),
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            ViewMode::Day => "D",
            ViewMode::Week => "W",
            ViewMode::Month => "M",
        }
    }
}
