use crate::widgets::markdown::core::{Node, Delta, node_factory::*};
use crate::widgets::markdown::core::block_keys::*;

/// Parse markdown text into nodes
pub struct MarkdownParser;

impl MarkdownParser {
    /// Detect block type from line and return node
    pub fn parse_line(line: &str) -> Node {
        let trimmed = line.trim_start();

        // Divider
        if matches!(trimmed, "---" | "***" | "___") {
            return divider_node();
        }

        // Headings (h1-h6)
        for level in (1..=6).rev() {
            let prefix = "#".repeat(level);
            if let Some(text) = trimmed.strip_prefix(&prefix).and_then(|s| s.strip_prefix(' ')) {
                return heading_node(level as u8, text);
            }
        }

        // Todo list
        if let Some(text) = trimmed.strip_prefix("- [ ] ") {
            return todo_list_node(text, false);
        }
        if let Some(text) = trimmed.strip_prefix("- [x] ") {
            return todo_list_node(text, true);
        }

        // Bulleted list
        if let Some(text) = trimmed.strip_prefix("- ") {
            return bulleted_list_node(text);
        }
        if let Some(text) = trimmed.strip_prefix("* ") {
            return bulleted_list_node(text);
        }

        // Numbered list
        if let Some((num, text)) = Self::parse_numbered_list(trimmed) {
            return numbered_list_node(num, text);
        }

        // Quote
        if let Some(text) = trimmed.strip_prefix("> ") {
            return quote_node(text);
        }

        // Code block start
        if trimmed.starts_with("```") {
            let lang = trimmed.strip_prefix("```").map(|s| s.trim().to_string());
            return code_node("", lang);
        }

        // Default: paragraph
        paragraph_node(line)
    }

    /// Parse numbered list (e.g., "1. text")
    fn parse_numbered_list(line: &str) -> Option<(u32, &str)> {
        let mut chars = line.chars();
        let mut num_str = String::new();

        while let Some(ch) = chars.next() {
            if ch.is_ascii_digit() {
                num_str.push(ch);
            } else if ch == '.' && !num_str.is_empty() {
                if chars.as_str().starts_with(' ') {
                    let num = num_str.parse().ok()?;
                    let text = chars.as_str().trim_start();
                    return Some((num, text));
                }
                return None;
            } else {
                return None;
            }
        }
        None
    }

    /// Detect if text starts with markdown marker and should transform
    pub fn detect_shortcut(text: &str) -> Option<(String, Node)> {
        let trimmed = text.trim();

        // Heading shortcuts
        for level in 1..=6 {
            let marker = format!("{} ", "#".repeat(level));
            if trimmed == marker.trim() {
                return Some((marker, heading_node(level as u8, "")));
            }
        }

        // List shortcuts
        if trimmed == "-" || trimmed == "- " {
            return Some(("- ".to_string(), bulleted_list_node("")));
        }

        if trimmed == ">" || trimmed == "> " {
            return Some(("> ".to_string(), quote_node("")));
        }

        // Todo shortcut
        if trimmed == "- [ ]" || trimmed == "- [ ] " {
            return Some(("- [ ] ".to_string(), todo_list_node("", false)));
        }

        // Numbered list shortcut
        if let Some((num, text)) = Self::parse_numbered_list(trimmed) {
            if text.is_empty() {
                let marker = format!("{}. ", num);
                return Some((marker, numbered_list_node(num, "")));
            }
        }

        // Divider
        if matches!(trimmed, "---" | "***" | "___") {
            return Some((trimmed.to_string(), divider_node()));
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heading_detection() {
        let node = MarkdownParser::parse_line("# Title");
        assert_eq!(node.node_type, heading::TYPE);
        assert_eq!(node.get_attribute(heading::LEVEL), Some(&1.into()));
    }

    #[test]
    fn test_list_detection() {
        let node = MarkdownParser::parse_line("- Item");
        assert_eq!(node.node_type, bulleted_list::TYPE);

        let node = MarkdownParser::parse_line("1. Item");
        assert_eq!(node.node_type, numbered_list::TYPE);
    }
}
