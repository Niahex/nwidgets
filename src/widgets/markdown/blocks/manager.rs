use gtk4::{self as gtk, prelude::*};
use gtk4::{Box, Orientation};
use std::rc::Rc;
use std::cell::RefCell;

use super::block::Block;
use super::block_type::BlockType;

/// Gestionnaire de blocs pour l'éditeur markdown
pub struct BlockManager {
    /// Container principal (vertical) contenant tous les blocs
    pub container: Box,

    /// Liste des blocs
    blocks: Rc<RefCell<Vec<Rc<RefCell<Block>>>>>,

    /// Référence à soi-même pour les événements
    self_ref: RefCell<Option<Rc<BlockManager>>>,
}

impl BlockManager {
    /// Crée un nouveau gestionnaire de blocs
    pub fn new() -> Self {
        let container = Box::new(Orientation::Vertical, 5);
        container.set_margin_start(20);
        container.set_margin_end(20);
        container.set_margin_top(20);
        container.set_margin_bottom(20);
        container.set_visible(true);
        container.set_vexpand(true);
        container.set_hexpand(true);

        Self {
            container,
            blocks: Rc::new(RefCell::new(Vec::new())),
            self_ref: RefCell::new(None),
        }
    }

    /// Initialise la référence circulaire
    pub fn init_self_ref(self: &Rc<Self>) {
        *self.self_ref.borrow_mut() = Some(self.clone());
    }

    /// Ajoute un bloc à la fin
    pub fn add_block(&self, block: Block) {
        let block = Rc::new(RefCell::new(block));
        self.container.append(&block.borrow().container);
        self.blocks.borrow_mut().push(block);
    }

    /// Insère un bloc à une position donnée
    pub fn insert_block(&self, index: usize, block: Block) {
        let block = Rc::new(RefCell::new(block));

        // Insérer dans le container visuel
        if index < self.blocks.borrow().len() {
            let next_block = &self.blocks.borrow()[index];
            self.container.insert_child_after(
                &block.borrow().container,
                Some(&next_block.borrow().container),
            );
        } else {
            self.container.append(&block.borrow().container);
        }

        // Insérer dans la liste
        self.blocks.borrow_mut().insert(index, block);
    }

    /// Supprime un bloc à une position donnée
    pub fn remove_block(&self, index: usize) -> Option<Block> {
        if index >= self.blocks.borrow().len() {
            return None;
        }

        let block = self.blocks.borrow_mut().remove(index);
        self.container.remove(&block.borrow().container);

        // Extraire le Block du Rc<RefCell<>>
        match Rc::try_unwrap(block) {
            Ok(cell) => Some(cell.into_inner()),
            Err(_) => None,
        }
    }

    /// Récupère le nombre de blocs
    pub fn block_count(&self) -> usize {
        self.blocks.borrow().len()
    }

    /// Récupère un bloc à une position donnée
    pub fn get_block(&self, index: usize) -> Option<Rc<RefCell<Block>>> {
        self.blocks.borrow().get(index).cloned()
    }

    /// Trouve l'index d'un bloc par son ID
    pub fn find_block_index(&self, id: &str) -> Option<usize> {
        self.blocks
            .borrow()
            .iter()
            .position(|b| b.borrow().id == id)
    }

    /// Charge du contenu markdown
    pub fn load_markdown(&self, content: &str) {
        // Vider les blocs existants
        self.clear();

        let manager_ref = self.self_ref.borrow().clone();

        // Parser le markdown en blocs
        for line in content.lines() {
            let block_type = BlockType::from_line(line);
            let block = Block::new_with_manager(block_type, line, manager_ref.clone());
            self.add_block(block);
        }

        // Ajouter un bloc paragraphe vide si aucun bloc
        if self.block_count() == 0 {
            let block = Block::new_with_manager(BlockType::Paragraph, "", manager_ref);
            self.add_block(block);
        }

        // Focus sur le premier bloc
        if self.block_count() > 0 {
            self.focus_block(0);
        }
    }

    /// Exporte tout le contenu en markdown
    pub fn export_markdown(&self) -> String {
        self.blocks
            .borrow()
            .iter()
            .map(|b| b.borrow().get_markdown())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Vide tous les blocs
    pub fn clear(&self) {
        let count = self.block_count();
        for _ in 0..count {
            self.remove_block(0);
        }
    }

    /// Focus sur un bloc spécifique
    pub fn focus_block(&self, index: usize) {
        if let Some(block) = self.get_block(index) {
            block.borrow().focus();
        }
    }

    /// Crée un nouveau bloc après le bloc courant
    pub fn create_block_after(&self, current_index: usize, block_type: BlockType) {
        let manager_ref = self.self_ref.borrow().clone();
        let new_block = Block::new_with_manager(block_type, "", manager_ref);
        self.insert_block(current_index + 1, new_block);
        self.focus_block(current_index + 1);
    }

    /// Fusionne deux blocs
    pub fn merge_blocks(&self, index1: usize, index2: usize) {
        if index1 >= self.block_count() || index2 >= self.block_count() {
            return;
        }

        let content1 = self.get_block(index1).unwrap().borrow().get_content();
        let content2 = self.get_block(index2).unwrap().borrow().get_content();
        let merged_content = format!("{}{}", content1, content2);

        self.get_block(index1)
            .unwrap()
            .borrow()
            .set_content(&merged_content);
        self.remove_block(index2);
    }
}
