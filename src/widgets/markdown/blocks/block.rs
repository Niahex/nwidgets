use gtk4::{self as gtk, prelude::*};
use gtk4::{Box, Orientation, TextBuffer, TextView, TextTag};
use std::sync::atomic::{AtomicU64, Ordering};
use std::rc::Rc;
use std::cell::RefCell;
use gtk4::pango;

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

    /// Flag pour éviter la récursion lors des modifications
    pub processing: Rc<RefCell<bool>>,
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
        let processing = Rc::new(RefCell::new(false));

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
                    processing,
                }
            }
            _ => {
                // Créer un buffer et un TextView pour les autres types
                let buffer = TextBuffer::new(None);

                // Ne strip le marqueur que si c'est du markdown brut (pas déjà formaté avec • ou numéros)
                let needs_stripping = match &block_type {
                    BlockType::BulletList => content.starts_with("- "),
                    BlockType::NumberedList(_) => content.starts_with(char::is_numeric) && content.contains(". ") && !content.starts_with("• "),
                    BlockType::Heading(_) => content.starts_with('#'),
                    BlockType::Toggle | BlockType::ToggleHeading(_) => content.starts_with('>'),
                    _ => false,
                };

                let final_content = if needs_stripping {
                    block_type.strip_marker(content)
                } else {
                    content
                };

                buffer.set_text(final_content);

                let text_view = TextView::builder()
                    .buffer(&buffer)
                    .wrap_mode(gtk::WrapMode::Word)
                    .editable(true)  // S'assurer que c'est éditable
                    .can_focus(true)  // Peut recevoir le focus
                    .focusable(true)
                    .margin_start(10)
                    .margin_end(10)
                    .margin_top(5)
                    .margin_bottom(5)
                    .pixels_above_lines(2)
                    .pixels_below_lines(2)
                    .height_request(30)
                    .width_request(200)  // Largeur minimale
                    .hexpand(true)
                    .vexpand(false)
                    .build();

                // Configuration supplémentaire
                text_view.set_visible(true);
                text_view.set_sensitive(true);  // Activé pour les interactions
                text_view.set_cursor_visible(true);  // Curseur visible

                // Log pour debug
                println!("[BLOCK] Created block {} - editable: {}, can_focus: {}",
                    id, text_view.is_editable(), text_view.can_focus());

                // Surveiller les changements du buffer pour détecter les marqueurs markdown
                let id_for_detection = id.clone();
                if let Some(mgr) = &manager {
                    let manager_for_detection = mgr.clone();
                    let processing_for_callback = processing.clone();

                    buffer.connect_changed(move |buf| {
                        // Éviter la récursion
                        if *processing_for_callback.borrow() {
                            return;
                        }

                        let (start, end) = buf.bounds();
                        let text = buf.text(&start, &end, false).to_string();

                        // Détecter si le texte commence par un marqueur markdown
                        let detected_type = BlockType::from_line(&text);
                        println!("[MARKDOWN] Text: '{}' -> Detected: {:?}", text, detected_type);

                        // Trouver l'index du bloc
                        if let Some(block_index) = manager_for_detection.find_block_index(&id_for_detection) {
                            if let Some(block) = manager_for_detection.get_block(block_index) {
                                let (should_transform, current_type) = {
                                    let block_borrowed = block.borrow();
                                    let current_type = block_borrowed.block_type.clone();
                                    println!("[MARKDOWN] Current type: {:?}, Detected: {:?}", current_type, detected_type);

                                    // Si le type détecté est différent du type actuel
                                    let should_transform = if detected_type != current_type {
                                        // Pour les paragraphes, permettre la transformation
                                        matches!(current_type, BlockType::Paragraph)
                                    } else {
                                        false
                                    };

                                    (should_transform, current_type)
                                };

                                if should_transform && !matches!(detected_type, BlockType::Paragraph) {
                                    println!("[MARKDOWN] Transforming to {:?}", detected_type);
                                    // Activer le flag de traitement
                                    *processing_for_callback.borrow_mut() = true;

                                    // Transformer le bloc
                                    let mut stripped = detected_type.strip_marker(&text).to_string();

                                    // Pour les listes, ajouter le préfixe visuel
                                    match &detected_type {
                                        BlockType::BulletList => {
                                            stripped = format!("• {}", stripped);
                                        }
                                        BlockType::NumberedList(num) => {
                                            stripped = format!("{}. {}", num, stripped);
                                        }
                                        _ => {}
                                    }

                                    println!("[MARKDOWN] Stripped text: '{}'", stripped);

                                    {
                                        let mut block_mut = block.borrow_mut();
                                        block_mut.block_type = detected_type.clone();

                                        // Réappliquer le style CSS
                                        if let Some(tv) = &block_mut.text_view {
                                            println!("[MARKDOWN] Applying CSS styling for {:?}", detected_type);
                                            Block::apply_block_styling(tv, &detected_type);
                                        }
                                    }

                                    // Mettre à jour le buffer avec le contenu sans marqueur
                                    buf.set_text(&stripped);

                                    // Appliquer le tag de style au buffer
                                    println!("[MARKDOWN] Applying text tag for {:?}", detected_type);
                                    Block::apply_text_tag(buf, &detected_type);

                                    // Positionner le curseur à la fin
                                    let end_iter = buf.end_iter();
                                    buf.place_cursor(&end_iter);

                                    // Désactiver le flag de traitement
                                    *processing_for_callback.borrow_mut() = false;
                                } else {
                                    // Vérifier si le bloc est devenu vide (seulement le préfixe)
                                    let should_become_paragraph = match &current_type {
                                        BlockType::BulletList => {
                                            text.trim().is_empty() || text.trim() == "•"
                                        }
                                        BlockType::NumberedList(_) => {
                                            let trimmed = text.trim();
                                            trimmed.is_empty() || trimmed.chars().all(|c| c.is_ascii_digit() || c == '.')
                                        }
                                        _ => false,
                                    };

                                    if should_become_paragraph {
                                        println!("[MARKDOWN] Empty list detected, converting to Paragraph");
                                        *processing_for_callback.borrow_mut() = true;

                                        // Transformer en paragraphe
                                        {
                                            let mut block_mut = block.borrow_mut();
                                            block_mut.block_type = BlockType::Paragraph;

                                            if let Some(tv) = &block_mut.text_view {
                                                Block::apply_block_styling(tv, &BlockType::Paragraph);
                                            }
                                        }

                                        // Vider le buffer
                                        buf.set_text("");

                                        *processing_for_callback.borrow_mut() = false;
                                    } else if !matches!(current_type, BlockType::Paragraph) {
                                        // Pas de transformation, mais réappliquer le tag pour maintenir le style
                                        *processing_for_callback.borrow_mut() = true;
                                        println!("[MARKDOWN] Maintaining tag for {:?}", current_type);
                                        Block::apply_text_tag(buf, &current_type);
                                        *processing_for_callback.borrow_mut() = false;
                                    }
                                }
                            }
                        }
                    });
                }

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

                if !placeholder.is_empty() && final_content.is_empty() {
                    text_view.set_tooltip_text(Some(placeholder));
                }

                // Appliquer les styles selon le type de bloc
                Self::apply_block_styling(&text_view, &block_type);

                // Configurer les événements si un gestionnaire est fourni
                if let Some(mgr) = manager {
                    events::setup_block_events(&text_view, id.clone(), mgr);
                }

                // Appliquer le tag initial si nécessaire
                Self::apply_text_tag(&buffer, &block_type);

                container.append(&text_view);

                Self {
                    block_type,
                    container,
                    buffer: Some(buffer),
                    text_view: Some(text_view),
                    id,
                    processing,
                }
            }
        }
    }

    /// Crée un tag de style pour un type de bloc
    fn create_text_tag(block_type: &BlockType) -> Option<TextTag> {
        let tag = match block_type {
            BlockType::Heading(1) => {
                let tag = TextTag::new(Some("h1"));
                tag.set_size(32 * pango::SCALE);
                tag.set_weight(700);
                tag.set_foreground(Some("#88c0d0"));
                Some(tag)
            }
            BlockType::Heading(2) => {
                let tag = TextTag::new(Some("h2"));
                tag.set_size(26 * pango::SCALE);
                tag.set_weight(600);
                tag.set_foreground(Some("#81a1c1"));
                Some(tag)
            }
            BlockType::Heading(3) => {
                let tag = TextTag::new(Some("h3"));
                tag.set_size(22 * pango::SCALE);
                tag.set_weight(600);
                tag.set_foreground(Some("#8fbcbb"));
                Some(tag)
            }
            BlockType::Heading(4) => {
                let tag = TextTag::new(Some("h4"));
                tag.set_size(18 * pango::SCALE);
                tag.set_weight(600);
                tag.set_foreground(Some("#5e81ac"));
                Some(tag)
            }
            BlockType::Heading(5) => {
                let tag = TextTag::new(Some("h5"));
                tag.set_size(16 * pango::SCALE);
                tag.set_weight(600);
                tag.set_foreground(Some("#5e81ac"));
                Some(tag)
            }
            BlockType::BulletList => {
                let tag = TextTag::new(Some("bullet"));
                tag.set_foreground(Some("#a3be8c"));
                Some(tag)
            }
            BlockType::NumberedList(_) => {
                let tag = TextTag::new(Some("numbered"));
                tag.set_foreground(Some("#eceff4"));
                Some(tag)
            }
            BlockType::CodeBlock(_) => {
                let tag = TextTag::new(Some("code"));
                tag.set_family(Some("JetBrains Mono, monospace"));
                tag.set_foreground(Some("#a3be8c"));
                tag.set_background(Some("#2e3440"));
                Some(tag)
            }
            _ => None,
        };
        tag
    }

    /// Applique le tag de style au buffer
    fn apply_text_tag(buffer: &TextBuffer, block_type: &BlockType) {
        // Retirer tous les tags existants
        let (start, end) = buffer.bounds();
        buffer.remove_all_tags(&start, &end);

        // Appliquer le nouveau tag si disponible
        if let Some(tag) = Self::create_text_tag(block_type) {
            let tag_table = buffer.tag_table();

            let tag_name = tag.name().map(|s| s.to_string()).unwrap_or_default();

            // Retirer l'ancien tag du même nom s'il existe
            if let Some(old_tag) = tag_table.lookup(&tag_name) {
                tag_table.remove(&old_tag);
            }

            // Ajouter le nouveau tag
            tag_table.add(&tag);

            // Appliquer le tag à tout le texte
            let (start, end) = buffer.bounds();
            buffer.apply_tag(&tag, &start, &end);
        }
    }

    /// Nettoie les anciennes classes CSS du TextView
    fn clear_block_styling(text_view: &TextView) {
        // Retirer toutes les classes CSS possibles
        for i in 1..=5 {
            text_view.remove_css_class(&format!("heading-{}", i));
        }
        text_view.remove_css_class("heading");
        text_view.remove_css_class("bullet-list");
        text_view.remove_css_class("numbered-list");
        text_view.remove_css_class("toggle");
        text_view.remove_css_class("code-block");
        text_view.remove_css_class("paragraph");
        text_view.set_left_margin(0);
    }

    /// Applique le style visuel au TextView selon le type de bloc
    pub fn apply_block_styling(text_view: &TextView, block_type: &BlockType) {
        // Nettoyer les anciens styles
        Self::clear_block_styling(text_view);

        // Appliquer les nouveaux styles
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

    /// Change le type du bloc (sans recréer le widget)
    pub fn change_type(&mut self, new_type: BlockType) {
        // Activer le flag pour éviter la récursion
        *self.processing.borrow_mut() = true;

        self.block_type = new_type.clone();

        // Réappliquer le style CSS si on a un TextView
        if let Some(tv) = &self.text_view {
            Self::apply_block_styling(tv, &new_type);
        }

        // Réappliquer le tag de texte si on a un buffer
        if let Some(buf) = &self.buffer {
            Self::apply_text_tag(buf, &new_type);
        }

        // Désactiver le flag
        *self.processing.borrow_mut() = false;
    }

    /// Focus sur le TextView du bloc
    pub fn focus(&self) {
        if let Some(text_view) = &self.text_view {
            text_view.grab_focus();
        }
    }
}
