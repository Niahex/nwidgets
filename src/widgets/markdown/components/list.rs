use gtk4::{TextBuffer, TextTag, TextTagTable};
use gtk4::prelude::*;

/// Crée les tags pour les listes
pub fn create_list_tags(tag_table: &TextTagTable) {
    // Liste à puces
    let list_tag = TextTag::builder()
        .name("list")
        .left_margin(20)
        .build();

    // Liste numérotée
    let numbered_list_tag = TextTag::builder()
        .name("numbered_list")
        .left_margin(30)
        .build();

    tag_table.add(&list_tag);
    tag_table.add(&numbered_list_tag);
}

/// Détecte si une ligne est une liste à puces
pub fn parse_bullet_list(line: &str) -> bool {
    let line_char_count = line.chars().count();
    line.starts_with("- ") && line_char_count > 2
}

/// Détecte si une ligne est une liste numérotée et retourne la longueur du marqueur
pub fn parse_numbered_list(line: &str) -> Option<usize> {
    let chars: Vec<char> = line.chars().collect();
    let mut num_len = 0;

    for (i, &ch) in chars.iter().enumerate() {
        if ch.is_ascii_digit() {
            num_len = i + 1;
        } else if ch == '.' && num_len > 0 && i + 1 < chars.len() && chars[i + 1] == ' ' {
            // Retourne la longueur du marqueur (chiffres + ". ")
            return Some(num_len + 2);
        } else {
            break;
        }
    }
    None
}

/// Applique le style de liste à puces
pub fn apply_bullet_list_style(
    buffer: &TextBuffer,
    tag_table: &TextTagTable,
    char_offset: usize,
    line_char_count: usize,
) {
    // Cacher "- "
    if let Some(hidden_tag) = tag_table.lookup("hidden") {
        let marker_start = buffer.iter_at_offset(char_offset as i32);
        let marker_end = buffer.iter_at_offset((char_offset + 2) as i32);
        buffer.apply_tag(&hidden_tag, &marker_start, &marker_end);
    }

    // Appliquer le style liste
    if let Some(list_tag) = tag_table.lookup("list") {
        let line_start = buffer.iter_at_offset(char_offset as i32);
        let line_end = buffer.iter_at_offset((char_offset + line_char_count) as i32);
        buffer.apply_tag(&list_tag, &line_start, &line_end);
    }
}

/// Applique le style de liste numérotée
pub fn apply_numbered_list_style(
    buffer: &TextBuffer,
    tag_table: &TextTagTable,
    char_offset: usize,
    line_char_count: usize,
    _marker_len: usize,
) {
    if let Some(numbered_list_tag) = tag_table.lookup("numbered_list") {
        let line_start = buffer.iter_at_offset(char_offset as i32);
        let line_end = buffer.iter_at_offset((char_offset + line_char_count) as i32);
        buffer.apply_tag(&numbered_list_tag, &line_start, &line_end);
    }
}
