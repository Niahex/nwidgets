use super::node::{Node, Delta};
use super::block_keys::*;

/// Factory functions for creating nodes (following AppFlowy pattern)

pub fn paragraph_node(text: impl Into<String>) -> Node {
    Node::new(paragraph::TYPE)
        .with_delta(Delta::new().insert(text))
}

pub fn heading_node(level: u8, text: impl Into<String>) -> Node {
    Node::new(heading::TYPE)
        .with_delta(Delta::new().insert(text))
        .with_attribute(heading::LEVEL, level.into())
}

pub fn bulleted_list_node(text: impl Into<String>) -> Node {
    Node::new(bulleted_list::TYPE)
        .with_delta(Delta::new().insert(text))
}

pub fn numbered_list_node(number: u32, text: impl Into<String>) -> Node {
    Node::new(numbered_list::TYPE)
        .with_delta(Delta::new().insert(text))
        .with_attribute(numbered_list::NUMBER, number.into())
}

pub fn todo_list_node(text: impl Into<String>, checked: bool) -> Node {
    Node::new(todo_list::TYPE)
        .with_delta(Delta::new().insert(text))
        .with_attribute(todo_list::CHECKED, checked.into())
}

pub fn quote_node(text: impl Into<String>) -> Node {
    Node::new(quote::TYPE)
        .with_delta(Delta::new().insert(text))
}

pub fn code_node(text: impl Into<String>, language: Option<String>) -> Node {
    let mut node = Node::new(code::TYPE)
        .with_delta(Delta::new().insert(text));

    if let Some(lang) = language {
        node = node.with_attribute(code::LANGUAGE, lang.into());
    }

    node
}

pub fn divider_node() -> Node {
    Node::new(divider::TYPE)
}
