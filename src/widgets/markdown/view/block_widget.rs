use gtk4::{self as gtk, prelude::*};
use gtk4::{Box, Orientation, TextView, TextBuffer, CheckButton};
use std::rc::Rc;
use std::cell::RefCell;

use crate::widgets::markdown::core::{Node, Delta};
use crate::widgets::markdown::core::block_keys::*;

/// Widget for rendering a single block
pub struct BlockWidget {
    pub container: Box,
    pub text_view: Option<TextView>,
    pub buffer: Option<TextBuffer>,
    node: Rc<RefCell<Node>>,
    processing: Rc<RefCell<bool>>,
}

impl BlockWidget {
    pub fn new(node: Node) -> Self {
        let container = Box::new(Orientation::Vertical, 0);
        container.set_margin_top(4);
        container.set_margin_bottom(4);

        let node_rc = Rc::new(RefCell::new(node.clone()));
        let processing = Rc::new(RefCell::new(false));

        let (text_view, buffer) = match node.node_type.as_str() {
            divider::TYPE => {
                let sep = gtk::Separator::new(Orientation::Horizontal);
                sep.set_margin_top(8);
                sep.set_margin_bottom(8);
                sep.add_css_class("markdown-divider");
                container.append(&sep);
                (None, None)
            }
            _ => {
                let (tv, buf) = Self::create_text_view(&node);
                container.append(&tv);
                (Some(tv), Some(buf))
            }
        };

        Self {
            container,
            text_view,
            buffer,
            node: node_rc,
            processing,
        }
    }

    fn create_text_view(node: &Node) -> (TextView, TextBuffer) {
        let buffer = TextBuffer::new(None);

        // Set initial text from delta
        if let Some(delta) = &node.delta {
            buffer.set_text(&delta.to_plain_text());
        }

        let text_view = TextView::builder()
            .buffer(&buffer)
            .wrap_mode(gtk::WrapMode::WordChar)
            .editable(true)
            .margin_start(12)
            .margin_end(12)
            .margin_top(2)
            .margin_bottom(2)
            .hexpand(true)
            .build();

        Self::apply_styling(&text_view, node);

        (text_view, buffer)
    }

    fn apply_styling(text_view: &TextView, node: &Node) {
        // Clear existing classes
        for class in ["paragraph", "heading", "quote", "code", "list"] {
            text_view.remove_css_class(class);
        }

        match node.node_type.as_str() {
            heading::TYPE => {
                text_view.add_css_class("heading");
                if let Some(level) = node.get_attribute(heading::LEVEL) {
                    if let Some(lvl) = level.as_u64() {
                        text_view.add_css_class(&format!("heading-{}", lvl));
                    }
                }
            }
            bulleted_list::TYPE | numbered_list::TYPE | todo_list::TYPE => {
                text_view.add_css_class("list");
                text_view.set_left_margin(24);
            }
            quote::TYPE => {
                text_view.add_css_class("quote");
                text_view.set_left_margin(16);
            }
            code::TYPE => {
                text_view.add_css_class("code");
            }
            _ => {
                text_view.add_css_class("paragraph");
            }
        }
    }

    pub fn get_text(&self) -> String {
        if let Some(buffer) = &self.buffer {
            let (start, end) = buffer.bounds();
            buffer.text(&start, &end, false).to_string()
        } else {
            String::new()
        }
    }

    pub fn set_text(&self, text: &str) {
        if let Some(buffer) = &self.buffer {
            *self.processing.borrow_mut() = true;
            buffer.set_text(text);
            *self.processing.borrow_mut() = false;
        }
    }

    pub fn update_node(&self, new_node: Node) {
        *self.node.borrow_mut() = new_node.clone();

        if let Some(tv) = &self.text_view {
            Self::apply_styling(tv, &new_node);
        }

        if let Some(delta) = &new_node.delta {
            self.set_text(&delta.to_plain_text());
        }
    }

    pub fn focus(&self) {
        if let Some(tv) = &self.text_view {
            tv.grab_focus();
        }
    }

    pub fn get_node(&self) -> Node {
        self.node.borrow().clone()
    }

    pub fn is_processing(&self) -> bool {
        *self.processing.borrow()
    }

    pub fn set_processing(&self, val: bool) {
        *self.processing.borrow_mut() = val;
    }
}
