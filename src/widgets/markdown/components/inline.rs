use gtk4::{TextBuffer, TextTag, TextTagTable};
use gtk4::prelude::*;
use gtk4::pango;

/// Crée les tags pour le formatage inline (gras, italique)
pub fn create_inline_tags(tag_table: &TextTagTable) {
    let bold_tag = TextTag::builder()
        .name("bold")
        .weight(700)
        .build();

    let italic_tag = TextTag::builder()
        .name("italic")
        .style(pango::Style::Italic)
        .build();

    tag_table.add(&bold_tag);
    tag_table.add(&italic_tag);
}

/// Parse et applique les styles inline (gras et italique)
pub fn apply_inline_styles(buffer: &TextBuffer, tag_table: &TextTagTable, text: &str) {
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut cursor = 0;

    while cursor < len {
        // Gras: **texte**
        if cursor + 3 < len && chars[cursor] == '*' && chars[cursor + 1] == '*' {
            let start_tag_pos = cursor;
            let mut end_tag_pos = start_tag_pos + 2;

            while end_tag_pos + 1 < len {
                if chars[end_tag_pos] == '*' && chars[end_tag_pos + 1] == '*' {
                    if end_tag_pos > start_tag_pos + 2 {
                        // Appliquer le tag gras
                        if let Some(bold_tag) = tag_table.lookup("bold") {
                            let start_iter = buffer.iter_at_offset((start_tag_pos + 2) as i32);
                            let end_iter = buffer.iter_at_offset(end_tag_pos as i32);
                            buffer.apply_tag(&bold_tag, &start_iter, &end_iter);
                        }

                        // Cacher les marqueurs **
                        if let Some(hidden_tag) = tag_table.lookup("hidden") {
                            let marker1_start = buffer.iter_at_offset(start_tag_pos as i32);
                            let marker1_end = buffer.iter_at_offset((start_tag_pos + 2) as i32);
                            buffer.apply_tag(&hidden_tag, &marker1_start, &marker1_end);

                            let marker2_start = buffer.iter_at_offset(end_tag_pos as i32);
                            let marker2_end = buffer.iter_at_offset((end_tag_pos + 2) as i32);
                            buffer.apply_tag(&hidden_tag, &marker2_start, &marker2_end);
                        }

                        cursor = end_tag_pos + 2;
                        break;
                    }
                }
                end_tag_pos += 1;
            }
            if end_tag_pos + 1 >= len {
                cursor += 1;
            }
            continue;
        }

        // Italique: *texte*
        if cursor + 2 < len && chars[cursor] == '*' {
            let start_tag_pos = cursor;
            let mut end_tag_pos = start_tag_pos + 1;

            while end_tag_pos < len {
                if chars[end_tag_pos] == '*' {
                    // Éviter de confondre avec **
                    if end_tag_pos + 1 < len && chars[end_tag_pos + 1] == '*' {
                        end_tag_pos += 1;
                        continue;
                    }

                    if end_tag_pos > start_tag_pos + 1 {
                        // Appliquer le tag italique
                        if let Some(italic_tag) = tag_table.lookup("italic") {
                            let start_iter = buffer.iter_at_offset((start_tag_pos + 1) as i32);
                            let end_iter = buffer.iter_at_offset(end_tag_pos as i32);
                            buffer.apply_tag(&italic_tag, &start_iter, &end_iter);
                        }

                        // Cacher les marqueurs *
                        if let Some(hidden_tag) = tag_table.lookup("hidden") {
                            let marker1_start = buffer.iter_at_offset(start_tag_pos as i32);
                            let marker1_end = buffer.iter_at_offset((start_tag_pos + 1) as i32);
                            buffer.apply_tag(&hidden_tag, &marker1_start, &marker1_end);

                            let marker2_start = buffer.iter_at_offset(end_tag_pos as i32);
                            let marker2_end = buffer.iter_at_offset((end_tag_pos + 1) as i32);
                            buffer.apply_tag(&hidden_tag, &marker2_start, &marker2_end);
                        }

                        cursor = end_tag_pos + 1;
                        break;
                    }
                }
                end_tag_pos += 1;
            }

            if end_tag_pos >= len {
                cursor += 1;
            }
            continue;
        }

        cursor += 1;
    }
}
