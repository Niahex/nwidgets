use gtk4::{self as gtk, prelude::*, glib, gdk};
use std::rc::Rc;
use std::cell::RefCell;

use super::block_type::BlockType;
use super::manager::BlockManager;

/// Configure les événements clavier pour un bloc
pub fn setup_block_events(
    text_view: &gtk::TextView,
    block_id: String,
    manager: Rc<BlockManager>,
) {
    let key_controller = gtk::EventControllerKey::new();
    let block_id_clone = block_id.clone();
    let manager_clone = manager.clone();

    key_controller.connect_key_pressed(move |_controller, key, _code, _modifier| {
        let block_index = if let Some(idx) = manager_clone.find_block_index(&block_id_clone) {
            idx
        } else {
            return glib::Propagation::Proceed;
        };

        match key {
            gdk::Key::Return => {
                // Créer un nouveau bloc après celui-ci
                handle_enter_key(&manager_clone, block_index);
                glib::Propagation::Stop
            }
            gdk::Key::BackSpace => {
                // Si le bloc est vide, le supprimer et fusionner avec le précédent
                if handle_backspace(&manager_clone, block_index) {
                    glib::Propagation::Stop
                } else {
                    glib::Propagation::Proceed
                }
            }
            gdk::Key::Up => {
                // Naviguer vers le bloc précédent si au début
                if handle_up_key(&manager_clone, block_index) {
                    glib::Propagation::Stop
                } else {
                    glib::Propagation::Proceed
                }
            }
            gdk::Key::Down => {
                // Naviguer vers le bloc suivant si à la fin
                if handle_down_key(&manager_clone, block_index) {
                    glib::Propagation::Stop
                } else {
                    glib::Propagation::Proceed
                }
            }
            _ => glib::Propagation::Proceed,
        }
    });

    text_view.add_controller(key_controller);
}

/// Gère la touche Entrée
fn handle_enter_key(manager: &BlockManager, current_index: usize) {
    // Déterminer le type du nouveau bloc
    let current_block = if let Some(block) = manager.get_block(current_index) {
        block
    } else {
        return;
    };

    let current_type = &current_block.borrow().block_type;
    let new_type = match current_type {
        BlockType::BulletList => {
            // Vérifier si le contenu est vide
            let content = current_block.borrow().get_content();
            if content.trim().is_empty() {
                // Transformer en paragraphe
                current_block.borrow_mut().change_type(BlockType::Paragraph);
                return;
            }
            BlockType::BulletList
        }
        BlockType::NumberedList(num) => {
            let content = current_block.borrow().get_content();
            if content.trim().is_empty() {
                current_block.borrow_mut().change_type(BlockType::Paragraph);
                return;
            }
            BlockType::NumberedList(num + 1)
        }
        BlockType::Heading(_) => BlockType::Paragraph,
        _ => BlockType::Paragraph,
    };

    manager.create_block_after(current_index, new_type);
}

/// Gère la touche Backspace
fn handle_backspace(manager: &BlockManager, current_index: usize) -> bool {
    let current_block = if let Some(block) = manager.get_block(current_index) {
        block
    } else {
        return false;
    };

    // Vérifier si le curseur est au début du bloc
    let buffer = if let Some(buf) = &current_block.borrow().buffer {
        buf.clone()
    } else {
        return false;
    };

    let cursor = buffer.iter_at_mark(&buffer.get_insert());
    let is_at_start = cursor.offset() == 0;

    if is_at_start && current_index > 0 {
        // Fusionner avec le bloc précédent
        manager.merge_blocks(current_index - 1, current_index);
        manager.focus_block(current_index - 1);
        return true;
    }

    false
}

/// Gère la touche Flèche Haut
fn handle_up_key(manager: &BlockManager, current_index: usize) -> bool {
    let current_block = if let Some(block) = manager.get_block(current_index) {
        block
    } else {
        return false;
    };

    let buffer = if let Some(buf) = &current_block.borrow().buffer {
        buf.clone()
    } else {
        return false;
    };

    let cursor = buffer.iter_at_mark(&buffer.get_insert());

    // Si on est à la première ligne, naviguer vers le bloc précédent
    if cursor.line() == 0 && current_index > 0 {
        manager.focus_block(current_index - 1);
        return true;
    }

    false
}

/// Gère la touche Flèche Bas
fn handle_down_key(manager: &BlockManager, current_index: usize) -> bool {
    let current_block = if let Some(block) = manager.get_block(current_index) {
        block
    } else {
        return false;
    };

    let buffer = if let Some(buf) = &current_block.borrow().buffer {
        buf.clone()
    } else {
        return false;
    };

    let cursor = buffer.iter_at_mark(&buffer.get_insert());
    let line_count = buffer.line_count();

    // Si on est à la dernière ligne, naviguer vers le bloc suivant
    if cursor.line() >= line_count - 1 && current_index < manager.block_count() - 1 {
        manager.focus_block(current_index + 1);
        return true;
    }

    false
}
