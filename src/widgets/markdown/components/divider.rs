use gtk4::{TextBuffer, TextTag, TextTagTable};
use gtk4::prelude::*;

/// Crée le tag pour les dividers
pub fn create_divider_tag(tag_table: &TextTagTable) {
    let divider_tag = TextTag::builder()
        .name("divider")
        .strikethrough(true)
        .foreground("gray")
        .build();

    tag_table.add(&divider_tag);
}

/// Détecte si une ligne est un divider horizontal
pub fn parse_divider(line: &str) -> bool {
    line.trim() == "---"
}

/// Applique le style de divider
pub fn apply_divider_style(
    buffer: &TextBuffer,
    tag_table: &TextTagTable,
    char_offset: usize,
    line_char_count: usize,
) {
    if let Some(divider_tag) = tag_table.lookup("divider") {
        let line_start = buffer.iter_at_offset(char_offset as i32);
        let line_end = buffer.iter_at_offset((char_offset + line_char_count) as i32);
        buffer.apply_tag(&divider_tag, &line_start, &line_end);
    }
}
