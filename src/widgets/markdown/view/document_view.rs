use gtk4::{self as gtk, prelude::*};
use gtk4::{Box, Orientation};
use std::rc::Rc;
use std::cell::RefCell;

use crate::widgets::markdown::core::{EditorState, Transaction, Node, Delta, node_factory};
use crate::widgets::markdown::parser::MarkdownParser;
use super::block_widget::BlockWidget;

/// Manages the visual representation of the document
pub struct DocumentView {
    pub container: Box,
    editor_state: Rc<EditorState>,
    block_widgets: Rc<RefCell<Vec<Rc<RefCell<BlockWidget>>>>>,
}

impl DocumentView {
    pub fn new() -> Self {
        let container = Box::new(Orientation::Vertical, 0);
        container.set_margin_start(16);
        container.set_margin_end(16);
        container.set_margin_top(16);
        container.set_margin_bottom(16);

        let editor_state = Rc::new(EditorState::new());
        let block_widgets = Rc::new(RefCell::new(Vec::new()));

        let view = Self {
            container,
            editor_state,
            block_widgets,
        };

        // Add initial paragraph
        view.insert_block(0, node_factory::paragraph_node(""));

        view
    }

    /// Insert a block at position
    pub fn insert_block(&self, index: usize, node: Node) {
        // Add to editor state
        let mut transaction = Transaction::new();
        transaction.insert_node(vec![index], node.clone());
        let _ = self.editor_state.apply(transaction);

        // Create widget
        let block_widget = Rc::new(RefCell::new(BlockWidget::new(node)));

        // Setup change detection
        if let Some(buffer) = block_widget.borrow().buffer.as_ref() {
            let bw_clone = block_widget.clone();
            let view_clone = Rc::new(self.clone_light());
            let idx = index;

            buffer.connect_changed(move |buf| {
                let widget = bw_clone.borrow();
                if widget.is_processing() {
                    return;
                }

                let text = {
                    let (start, end) = buf.bounds();
                    buf.text(&start, &end, false).to_string()
                };

                // Check for markdown shortcuts
                if let Some((marker, new_node)) = MarkdownParser::detect_shortcut(&text) {
                    widget.set_processing(true);

                    // Transform block
                    let mut node_with_content = new_node.clone();
                    if let Some(delta) = node_with_content.delta.as_mut() {
                        *delta = Delta::new().insert(text.trim_start_matches(marker.as_str()));
                    }

                    widget.update_node(node_with_content);

                    // Update cursor position
                    let end_iter = buf.end_iter();
                    buf.place_cursor(&end_iter);

                    widget.set_processing(false);
                }
            });
        }

        // Add to widget list and container
        let mut widgets = self.block_widgets.borrow_mut();
        widgets.insert(index, block_widget.clone());

        if index < widgets.len() - 1 {
            let next = &widgets[index + 1];
            self.container.insert_child_after(
                &block_widget.borrow().container,
                Some(&next.borrow().container)
            );
        } else {
            self.container.append(&block_widget.borrow().container);
        }

        drop(widgets);

        // Focus new block
        block_widget.borrow().focus();
    }

    /// Delete block at position
    pub fn delete_block(&self, index: usize) {
        let mut widgets = self.block_widgets.borrow_mut();
        if index >= widgets.len() {
            return;
        }

        let widget = widgets.remove(index);
        self.container.remove(&widget.borrow().container);

        let mut transaction = Transaction::new();
        transaction.delete_node(vec![index]);
        let _ = self.editor_state.apply(transaction);

        // Focus previous or next block
        if !widgets.is_empty() {
            let focus_idx = if index > 0 { index - 1 } else { 0 };
            if let Some(w) = widgets.get(focus_idx) {
                w.borrow().focus();
            }
        }
    }

    /// Move block from one position to another
    pub fn move_block(&self, from: usize, to: usize) {
        let mut widgets = self.block_widgets.borrow_mut();
        if from >= widgets.len() || to > widgets.len() {
            return;
        }

        let widget = widgets.remove(from);
        widgets.insert(to, widget.clone());

        self.container.remove(&widget.borrow().container);

        if to < widgets.len() - 1 {
            self.container.insert_child_after(
                &widget.borrow().container,
                Some(&widgets[to + 1].borrow().container)
            );
        } else {
            self.container.append(&widget.borrow().container);
        }

        let mut transaction = Transaction::new();
        transaction.move_node(vec![from], vec![to]);
        let _ = self.editor_state.apply(transaction);
    }

    /// Get block count
    pub fn block_count(&self) -> usize {
        self.block_widgets.borrow().len()
    }

    /// Load markdown text
    pub fn load_markdown(&self, text: &str) {
        // Clear existing blocks
        let widgets = self.block_widgets.borrow().clone();
        for widget in widgets.iter() {
            self.container.remove(&widget.borrow().container);
        }
        self.block_widgets.borrow_mut().clear();

        // Parse and add blocks
        for (i, line) in text.lines().enumerate() {
            let node = MarkdownParser::parse_line(line);
            self.insert_block(i, node);
        }

        // Add initial block if empty
        if self.block_count() == 0 {
            self.insert_block(0, node_factory::paragraph_node(""));
        }
    }

    /// Export to markdown
    pub fn export_markdown(&self) -> String {
        let widgets = self.block_widgets.borrow();
        widgets.iter()
            .map(|w| {
                let widget = w.borrow();
                let node = widget.get_node();
                self.node_to_markdown(&node)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn node_to_markdown(&self, node: &Node) -> String {
        use crate::widgets::markdown::core::block_keys::*;

        let text = node.delta.as_ref()
            .map(|d| d.to_plain_text())
            .unwrap_or_default();

        match node.node_type.as_str() {
            paragraph::TYPE => text,
            heading::TYPE => {
                let level = node.get_attribute(heading::LEVEL)
                    .and_then(|v| v.as_u64())
                    .unwrap_or(1) as usize;
                format!("{} {}", "#".repeat(level), text)
            }
            bulleted_list::TYPE => format!("- {}", text),
            numbered_list::TYPE => {
                let num = node.get_attribute(numbered_list::NUMBER)
                    .and_then(|v| v.as_u64())
                    .unwrap_or(1);
                format!("{}. {}", num, text)
            }
            todo_list::TYPE => {
                let checked = node.get_attribute(todo_list::CHECKED)
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let marker = if checked { "[x]" } else { "[ ]" };
                format!("- {} {}", marker, text)
            }
            quote::TYPE => format!("> {}", text),
            code::TYPE => {
                let lang = node.get_attribute(code::LANGUAGE)
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                format!("```{}\n{}\n```", lang, text)
            }
            divider::TYPE => "---".to_string(),
            _ => text,
        }
    }

    fn clone_light(&self) -> Self {
        Self {
            container: self.container.clone(),
            editor_state: self.editor_state.clone(),
            block_widgets: self.block_widgets.clone(),
        }
    }
}
