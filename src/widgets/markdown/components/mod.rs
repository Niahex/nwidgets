pub mod heading;
pub mod list;
pub mod toggle;
pub mod divider;
pub mod inline;

use gtk4::{TextBuffer, TextTag, TextTagTable};
use gtk4::prelude::*;

/// Crée le tag "hidden" utilisé pour cacher les marqueurs markdown
pub fn create_hidden_tag(tag_table: &TextTagTable) {
    let hidden_tag = TextTag::builder()
        .name("hidden")
        .invisible(true)
        .build();

    tag_table.add(&hidden_tag);
}

/// Crée tous les tags nécessaires pour le rendu markdown
pub fn create_all_tags(tag_table: &TextTagTable) {
    heading::create_heading_tags(tag_table);
    list::create_list_tags(tag_table);
    toggle::create_toggle_tag(tag_table);
    divider::create_divider_tag(tag_table);
    inline::create_inline_tags(tag_table);
    create_hidden_tag(tag_table);
}

/// Supprime tous les tags du buffer
pub fn remove_all_tags(buffer: &TextBuffer, tag_table: &TextTagTable) {
    let (start_iter, end_iter) = buffer.bounds();

    let tag_names = [
        "h1", "h2", "h3", "h4", "h5",
        "bold", "italic",
        "list", "numbered_list",
        "toggle", "divider",
        "hidden"
    ];

    for tag_name in tag_names {
        if let Some(tag) = tag_table.lookup(tag_name) {
            buffer.remove_tag(&tag, &start_iter, &end_iter);
        }
    }
}
