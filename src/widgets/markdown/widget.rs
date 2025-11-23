use gtk4 as gtk;
use gtk4::prelude::*;
use gtk4::{ScrolledWindow, CssProvider};
use gtk4::gdk::Display;
use std::rc::Rc;
use std::cell::RefCell;

use super::blocks::{BlockManager, BlockType, Block};

pub struct MarkdownWidget {
    pub container: ScrolledWindow,
    block_manager: Rc<BlockManager>,
}

impl MarkdownWidget {
    pub fn new() -> Self {
        // Charger le CSS
        Self::load_css();

        let block_manager = Rc::new(BlockManager::new());

        // Initialiser la référence circulaire
        block_manager.init_self_ref();

        // Ajouter un bloc paragraphe initial
        let initial_block = Block::new_with_manager(
            BlockType::Paragraph,
            "",
            Some(block_manager.clone()),
        );
        block_manager.add_block(initial_block);

        // Focus sur le premier bloc
        block_manager.focus_block(0);

        let scrolled_window = ScrolledWindow::builder()
            .child(&block_manager.container)
            .build();

        // Ajouter des classes CSS pour un meilleur contrôle
        scrolled_window.add_css_class("markdown-editor");
        block_manager.container.add_css_class("markdown-container");

        Self {
            container: scrolled_window,
            block_manager,
        }
    }

    /// Charge le CSS pour les blocs
    fn load_css() {
        let css = include_str!("blocks/style.css");
        let provider = CssProvider::new();
        provider.load_from_data(css);

        gtk::style_context_add_provider_for_display(
            &Display::default().expect("Could not connect to a display."),
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    /// Charge du contenu markdown
    pub fn set_text(&self, text: &str) {
        self.block_manager.load_markdown(text);
    }

    /// Récupère le contenu markdown
    pub fn get_text(&self) -> String {
        self.block_manager.export_markdown()
    }

    /// Ajoute un nouveau bloc
    pub fn add_block(&self, block_type: BlockType, content: &str) {
        let block = Block::new(block_type, content);
        self.block_manager.add_block(block);
    }

    /// Récupère le gestionnaire de blocs
    pub fn get_block_manager(&self) -> &Rc<BlockManager> {
        &self.block_manager
    }
}
