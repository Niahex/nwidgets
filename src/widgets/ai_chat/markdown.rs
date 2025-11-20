use gpui::*;
use pulldown_cmark::{Parser, Event, Tag, TagEnd, HeadingLevel};
use crate::theme::*;

/// Convertit du markdown en éléments GPUI
pub fn render_markdown(content: &str) -> impl IntoElement {
    let parser = Parser::new(content);
    let mut elements: Vec<AnyElement> = Vec::new();
    let mut current_text = String::new();
    let mut in_code_block = false;
    let mut code_block_content = String::new();
    let mut in_bold = false;
    let mut in_italic = false;
    let mut in_code = false;
    let mut heading_level: Option<HeadingLevel> = None;
    let mut in_list = false;

    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                Tag::Heading { level, .. } => {
                    if !current_text.is_empty() {
                        elements.push(create_text_element(&current_text, in_bold, in_italic, in_code, None).into_any_element());
                        current_text.clear();
                    }
                    heading_level = Some(level);
                }
                Tag::Strong => {
                    in_bold = true;
                }
                Tag::Emphasis => {
                    in_italic = true;
                }
                Tag::CodeBlock(_) => {
                    if !current_text.is_empty() {
                        elements.push(create_text_element(&current_text, in_bold, in_italic, in_code, heading_level).into_any_element());
                        current_text.clear();
                    }
                    in_code_block = true;
                    code_block_content.clear();
                }
                Tag::List(_) => {
                    if !current_text.is_empty() {
                        elements.push(create_text_element(&current_text, in_bold, in_italic, in_code, heading_level).into_any_element());
                        current_text.clear();
                    }
                    in_list = true;
                }
                Tag::Item => {
                    if in_list && !current_text.is_empty() {
                        elements.push(create_list_item(&current_text).into_any_element());
                        current_text.clear();
                    }
                }
                Tag::Paragraph => {
                    if !current_text.is_empty() {
                        elements.push(create_text_element(&current_text, in_bold, in_italic, in_code, heading_level).into_any_element());
                        current_text.clear();
                    }
                }
                Tag::Link { dest_url, .. } => {
                    // Pour l'instant on affiche juste le lien comme du texte
                    current_text.push_str(&format!(" [{}]", dest_url));
                }
                _ => {}
            },
            Event::End(tag_end) => match tag_end {
                TagEnd::Heading(_) => {
                    if !current_text.is_empty() {
                        elements.push(create_text_element(&current_text, in_bold, in_italic, in_code, heading_level).into_any_element());
                        current_text.clear();
                    }
                    heading_level = None;
                }
                TagEnd::Strong => {
                    in_bold = false;
                }
                TagEnd::Emphasis => {
                    in_italic = false;
                }
                TagEnd::CodeBlock => {
                    if in_code_block {
                        elements.push(create_code_block(&code_block_content).into_any_element());
                        code_block_content.clear();
                        in_code_block = false;
                    }
                }
                TagEnd::Paragraph => {
                    if !current_text.is_empty() {
                        elements.push(create_text_element(&current_text, in_bold, in_italic, in_code, heading_level).into_any_element());
                        current_text.clear();
                    }
                }
                TagEnd::List(_) => {
                    if !current_text.is_empty() {
                        elements.push(create_list_item(&current_text).into_any_element());
                        current_text.clear();
                    }
                    in_list = false;
                }
                TagEnd::Item => {
                    if in_list && !current_text.is_empty() {
                        elements.push(create_list_item(&current_text).into_any_element());
                        current_text.clear();
                    }
                }
                _ => {}
            },
            Event::Text(text) => {
                if in_code_block {
                    code_block_content.push_str(&text);
                } else {
                    current_text.push_str(&text);
                }
            }
            Event::Code(code) => {
                if !current_text.is_empty() {
                    elements.push(create_text_element(&current_text, in_bold, in_italic, false, heading_level).into_any_element());
                    current_text.clear();
                }
                elements.push(create_inline_code(&code).into_any_element());
            }
            Event::SoftBreak | Event::HardBreak => {
                current_text.push(' ');
            }
            _ => {}
        }
    }

    // Ajouter le texte restant
    if !current_text.is_empty() {
        elements.push(create_text_element(&current_text, in_bold, in_italic, in_code, heading_level).into_any_element());
    }

    div()
        .flex()
        .flex_col()
        .gap_2()
        .children(elements)
}

fn create_text_element(
    text: &str,
    bold: bool,
    italic: bool,
    code: bool,
    heading: Option<HeadingLevel>,
) -> impl IntoElement {
    let mut element = div().text_sm().text_color(rgb(SNOW1));

    // Heading
    if let Some(level) = heading {
        element = match level {
            HeadingLevel::H1 => element.text_2xl().font_weight(FontWeight::BOLD).mb_2(),
            HeadingLevel::H2 => element.text_xl().font_weight(FontWeight::BOLD).mb_2(),
            HeadingLevel::H3 => element.text_lg().font_weight(FontWeight::SEMIBOLD).mb_1(),
            _ => element.text_base().font_weight(FontWeight::SEMIBOLD).mb_1(),
        };
    }

    // Style
    if bold {
        element = element.font_weight(FontWeight::BOLD);
    }
    if italic {
        // GPUI doesn't have italic directly, could use a different color
        element = element.text_color(rgb(FROST2));
    }
    if code {
        element = element.px_1().bg(rgb(POLAR3)).rounded_sm().text_color(rgb(FROST1));
    }

    element.child(text.to_string())
}

fn create_code_block(code: &str) -> impl IntoElement {
    div()
        .w_full()
        .p_3()
        .bg(rgb(POLAR3))
        .rounded_md()
        .text_sm()
        .text_color(rgb(SNOW0))
        .font_family("monospace")
        .child(code.to_string())
}

fn create_inline_code(code: &str) -> impl IntoElement {
    div()
        .px_1()
        .bg(rgb(POLAR3))
        .rounded_sm()
        .text_sm()
        .text_color(rgb(FROST1))
        .font_family("monospace")
        .child(code.to_string())
}

fn create_list_item(text: &str) -> impl IntoElement {
    div()
        .flex()
        .gap_2()
        .child(
            div()
                .text_sm()
                .text_color(rgb(FROST1))
                .child("•")
        )
        .child(
            div()
                .text_sm()
                .text_color(rgb(SNOW1))
                .child(text.to_string())
        )
}
