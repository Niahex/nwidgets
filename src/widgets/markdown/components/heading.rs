use gtk4::{TextBuffer, TextTag, TextTagTable};
use gtk4::prelude::*;

/// Crée les tags pour les titres h1-h5
pub fn create_heading_tags(tag_table: &TextTagTable) {
    let h1_tag = TextTag::builder()
        .name("h1")
        .weight(700)
        .scale(1.8)
        .pixels_above_lines(15)
        .pixels_below_lines(10)
        .build();

    let h2_tag = TextTag::builder()
        .name("h2")
        .weight(700)
        .scale(1.5)
        .pixels_above_lines(12)
        .pixels_below_lines(8)
        .build();

    let h3_tag = TextTag::builder()
        .name("h3")
        .weight(700)
        .scale(1.3)
        .pixels_above_lines(10)
        .pixels_below_lines(6)
        .build();

    let h4_tag = TextTag::builder()
        .name("h4")
        .weight(700)
        .scale(1.2)
        .pixels_above_lines(8)
        .pixels_below_lines(4)
        .build();

    let h5_tag = TextTag::builder()
        .name("h5")
        .weight(700)
        .scale(1.1)
        .pixels_above_lines(6)
        .pixels_below_lines(3)
        .build();

    tag_table.add(&h1_tag);
    tag_table.add(&h2_tag);
    tag_table.add(&h3_tag);
    tag_table.add(&h4_tag);
    tag_table.add(&h5_tag);
}

/// Détecte si une ligne est un titre et retourne (niveau, longueur_marqueur)
pub fn parse_heading(line: &str) -> Option<(u8, usize)> {
    let line_char_count = line.chars().count();

    if line.starts_with("##### ") && line_char_count > 6 {
        Some((5, 6))
    } else if line.starts_with("#### ") && line_char_count > 5 {
        Some((4, 5))
    } else if line.starts_with("### ") && line_char_count > 4 {
        Some((3, 4))
    } else if line.starts_with("## ") && line_char_count > 3 {
        Some((2, 3))
    } else if line.starts_with("# ") && line_char_count > 2 {
        Some((1, 2))
    } else {
        None
    }
}

/// Applique le style de titre à une ligne
pub fn apply_heading_style(
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

    // Cacher le marqueur (# ## ### etc)
    if let Some(hidden_tag) = tag_table.lookup("hidden") {
        let marker_start = buffer.iter_at_offset(char_offset as i32);
        let marker_end = buffer.iter_at_offset((char_offset + marker_len) as i32);
        buffer.apply_tag(&hidden_tag, &marker_start, &marker_end);
    }

    // Appliquer le style au contenu
    if let Some(header_tag) = tag_table.lookup(&tag_name) {
        let content_start = buffer.iter_at_offset((char_offset + marker_len) as i32);
        let content_end = buffer.iter_at_offset((char_offset + line_char_count) as i32);
        buffer.apply_tag(&header_tag, &content_start, &content_end);
    }
}
