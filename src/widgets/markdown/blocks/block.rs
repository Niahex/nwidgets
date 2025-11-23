use gtk4::{self as gtk, prelude::*};
use gtk4::{Box, Orientation, TextBuffer, TextView};
use std::sync::atomic::{AtomicU64, Ordering};
use std::rc::Rc;

use super::block_type::BlockType;
use super::manager::BlockManager;
use super::events;

static BLOCK_ID_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Représente un bloc dans l'éditeur markdown
pub struct Block {
    /// Type du bloc
    pub block_type: BlockType,

    /// Container principal du bloc
    pub container: Box,

    /// Buffer de texte pour les blocs éditables
    pub buffer: Option<TextBuffer>,

    /// TextView pour les blocs éditables
    pub text_view: Option<TextView>,

    /// ID unique du bloc
    pub id: String,
}

impl Block {
    /// Crée un nouveau bloc
    pub fn new(block_type: BlockType, content: &str) -> Self {
        Self::new_with_manager(block_type, content, None)
    }

    /// Crée un nouveau bloc avec un gestionnaire (pour les événements)
    pub fn new_with_manager(
        block_type: BlockType,
        content: &str,
        manager: Option<Rc<BlockManager>>,
    ) -> Self {
        let id = BLOCK_ID_COUNTER.fetch_add(1, Ordering::SeqCst).to_string();
        let container = Box::new(Orientation::Horizontal, 0);
        container.add_css_class("markdown-block");
        container.set_visible(true);
        container.set_vexpand(false);
        container.set_hexpand(true);

        match &block_type {
            BlockType::Divider => {
                let separator = gtk::Separator::new(Orientation::Horizontal);
                separator.set_margin_top(10);
                separator.set_margin_bottom(10);
                container.append(&separator);

                Self {
                    block_type,
                    container,
                    buffer: None,
                    text_view: None,
                    id,
                }
            }
            _ => {
                // Créer un buffer et un TextView pour les autres types
                let buffer = TextBuffer::new(None);
                let stripped_content = block_type.strip_marker(content);
                buffer.set_text(stripped_content);

                let text_view = TextView::builder()
                    .buffer(&buffer)
                    .wrap_mode(gtk::WrapMode::Word)
                    .margin_start(10)
                    .margin_end(10)
                    .margin_top(5)
                    .margin_bottom(5)
                    .pixels_above_lines(2)
                    .pixels_below_lines(2)
                    .height_request(30)  // Hauteur minimale pour voir le bloc
                    .build();

                // Rendre le TextView visible
                text_view.set_visible(true);

                // Ajouter un placeholder via tooltip (temporaire)
                let placeholder = match &block_type {
                    BlockType::Paragraph => "Tapez '/' pour les commandes...",
                    BlockType::Heading(1) => "Titre 1",
                    BlockType::Heading(2) => "Titre 2",
                    BlockType::Heading(3) => "Titre 3",
                    BlockType::Heading(_) => "Titre",
                    BlockType::BulletList => "Liste",
                    BlockType::NumberedList(_) => "Liste numérotée",
                    _ => "",
                };

                if !placeholder.is_empty() && stripped_content.is_empty() {
                    text_view.set_tooltip_text(Some(placeholder));
                }

                // Appliquer les styles selon le type de bloc
                Self::apply_block_styling(&text_view, &block_type);

                // Configurer les événements si un gestionnaire est fourni
                if let Some(mgr) = manager {
                    events::setup_block_events(&text_view, id.clone(), mgr);
                }

                container.append(&text_view);

                Self {
                    block_type,
                    container,
                    buffer: Some(buffer),
                    text_view: Some(text_view),
                    id,
                }
            }
        }
    }

    /// Applique le style visuel au TextView selon le type de bloc
    fn apply_block_styling(text_view: &TextView, block_type: &BlockType) {
        match block_type {
            BlockType::Heading(level) => {
                text_view.add_css_class(&format!("heading-{}", level));
                text_view.add_css_class("heading");
            }
            BlockType::BulletList => {
                text_view.add_css_class("bullet-list");
                text_view.set_left_margin(30);
            }
            BlockType::NumberedList(_) => {
                text_view.add_css_class("numbered-list");
                text_view.set_left_margin(40);
            }
            BlockType::Toggle | BlockType::ToggleHeading(_) => {
                text_view.add_css_class("toggle");
                text_view.set_left_margin(30);
            }
            BlockType::CodeBlock(_) => {
                text_view.add_css_class("code-block");
            }
            BlockType::Paragraph => {
                text_view.add_css_class("paragraph");
            }
            BlockType::Divider => {}
        }
    }

    /// Récupère le contenu du bloc
    pub fn get_content(&self) -> String {
        if let Some(buffer) = &self.buffer {
            let (start, end) = buffer.bounds();
            buffer.text(&start, &end, false).to_string()
        } else {
            String::new()
        }
    }

    /// Définit le contenu du bloc
    pub fn set_content(&self, content: &str) {
        if let Some(buffer) = &self.buffer {
            buffer.set_text(content);
        }
    }

    /// Récupère le texte complet avec marqueur markdown
    pub fn get_markdown(&self) -> String {
        let content = self.get_content();

        match &self.block_type {
            BlockType::Paragraph => content,
            BlockType::Heading(level) => {
                format!("{} {}", "#".repeat(*level as usize), content)
            }
            BlockType::BulletList => {
                format!("- {}", content)
            }
            BlockType::NumberedList(num) => {
                format!("{}. {}", num, content)
            }
            BlockType::Toggle => {
                format!("> {}", content)
            }
            BlockType::ToggleHeading(level) => {
                format!(">{} {}", "#".repeat(*level as usize), content)
            }
            BlockType::Divider => "---".to_string(),
            BlockType::CodeBlock(lang) => {
                if let Some(l) = lang {
                    format!("```{}\n{}\n```", l, content)
                } else {
                    format!("```\n{}\n```", content)
                }
            }
        }
    }

    /// Change le type du bloc
    pub fn change_type(&mut self, new_type: BlockType) {
        let content = self.get_content();
        *self = Self::new(new_type, &content);
    }

    /// Focus sur le TextView du bloc
    pub fn focus(&self) {
        if let Some(text_view) = &self.text_view {
            text_view.grab_focus();
        }
    }
}
