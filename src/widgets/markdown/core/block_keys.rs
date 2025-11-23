/// Block type constants following AppFlowy pattern
pub mod paragraph {
    pub const TYPE: &str = "paragraph";
    pub const DELTA: &str = "delta";
}

pub mod heading {
    pub const TYPE: &str = "heading";
    pub const LEVEL: &str = "level";
    pub const DELTA: &str = "delta";
}

pub mod bulleted_list {
    pub const TYPE: &str = "bulleted_list";
    pub const DELTA: &str = "delta";
}

pub mod numbered_list {
    pub const TYPE: &str = "numbered_list";
    pub const DELTA: &str = "delta";
    pub const NUMBER: &str = "number";
}

pub mod todo_list {
    pub const TYPE: &str = "todo_list";
    pub const DELTA: &str = "delta";
    pub const CHECKED: &str = "checked";
}

pub mod quote {
    pub const TYPE: &str = "quote";
    pub const DELTA: &str = "delta";
}

pub mod code {
    pub const TYPE: &str = "code";
    pub const DELTA: &str = "delta";
    pub const LANGUAGE: &str = "language";
}

pub mod divider {
    pub const TYPE: &str = "divider";
}
