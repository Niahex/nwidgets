use gtk4::{TextBuffer, TextTag, TextTagTable};
use gtk4::prelude::*;

/// Crée le tag pour les toggles
pub fn create_toggle_tag(tag_table: &TextTagTable) {
    let toggle_tag = TextTag::builder()
        .name("toggle")
        .left_margin(20)
        .build();

    tag_table.add(&toggle_tag);
}

/// Détecte si une ligne est un toggle simple
pub fn parse_toggle(line: &str) -> bool {
    let line_char_count = line.chars().count();
    line.starts_with("> ") && line_char_count > 2
}

/// Détecte si une ligne est un toggle avec titre et retourne (niveau, longueur_marqueur)
pub fn parse_toggle_heading(line: &str) -> Option<(u8, usize)> {
    let line_char_count = line.chars().count();

    if line.starts_with(">##### ") && line_char_count > 7 {
        Some((5, 7))
    } else if line.starts_with(">#### ") && line_char_count > 6 {
        Some((4, 6))
    } else if line.starts_with(">### ") && line_char_count > 5 {
        Some((3, 5))
    } else if line.starts_with(">## ") && line_char_count > 4 {
        Some((2, 4))
    } else if line.starts_with("># ") && line_char_count > 3 {
        Some((1, 3))
    } else {
        None
    }
}

/// Applique le style de toggle simple
pub fn apply_toggle_style(
    buffer: &TextBuffer,
    tag_table: &TextTagTable,
    char_offset: usize,
    line_char_count: usize,
) {
    // Cacher "> "
    if let Some(hidden_tag) = tag_table.lookup("hidden") {
        let marker_start = buffer.iter_at_offset(char_offset as i32);
        let marker_end = buffer.iter_at_offset((char_offset + 2) as i32);
        buffer.apply_tag(&hidden_tag, &marker_start, &marker_end);
    }

    // Appliquer le style toggle
    if let Some(toggle_tag) = tag_table.lookup("toggle") {
        let line_start = buffer.iter_at_offset(char_offset as i32);
        let line_end = buffer.iter_at_offset((char_offset + line_char_count) as i32);
        buffer.apply_tag(&toggle_tag, &line_start, &line_end);
    }
}

/// Applique le style de toggle avec titre
pub fn apply_toggle_heading_style(
    buffer: &TextBuffer,
    tag_table: &TextTagTable,
    char_offset: usize,
    line_char_count: usize,
    level: u8,
    marker_len: usize,
) {
    if char_offset + line_char_count > buffer.char_count() as usize || marker_len >= line_char_count {
        return;
    }

    let tag_name = format!("h{}", level);

    // Cacher le marqueur du toggle (le ">")
    if let Some(hidden_tag) = tag_table.lookup("hidden") {
        let marker_start = buffer.iter_at_offset(char_offset as i32);
        let marker_end = buffer.iter_at_offset((char_offset + 1) as i32);
        buffer.apply_tag(&hidden_tag, &marker_start, &marker_end);
    }

    // Appliquer le style toggle
    if let Some(toggle_tag) = tag_table.lookup("toggle") {
        let line_start = buffer.iter_at_offset(char_offset as i32);
        let line_end = buffer.iter_at_offset((char_offset + line_char_count) as i32);
        buffer.apply_tag(&toggle_tag, &line_start, &line_end);
    }

    // Cacher le marqueur de titre (# ## ### etc)
    if let Some(hidden_tag) = tag_table.lookup("hidden") {
        let header_marker_start = buffer.iter_at_offset((char_offset + 1) as i32);
        let header_marker_end = buffer.iter_at_offset((char_offset + marker_len) as i32);
        buffer.apply_tag(&hidden_tag, &header_marker_start, &header_marker_end);
    }

    // Appliquer le style de titre au contenu
    if let Some(header_tag) = tag_table.lookup(&tag_name) {
        let content_start = buffer.iter_at_offset((char_offset + marker_len) as i32);
        let content_end = buffer.iter_at_offset((char_offset + line_char_count) as i32);
        buffer.apply_tag(&header_tag, &content_start, &content_end);
    }
}
