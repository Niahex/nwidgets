use crate::theme::ActiveTheme;
use gpui::prelude::*;
use gpui::*;

struct Block {
    id: usize,
    content: SharedString,
    rendered: SharedString,
    is_editing: bool,
}

pub struct MarkdownWidget {
    blocks: Vec<Block>,
    next_id: usize,
    focus_handle: FocusHandle,
}

impl MarkdownWidget {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            blocks: vec![Block {
                id: 0,
                content: "# Welcome\n\nClick to edit".into(),
                rendered: Self::parse_markdown("# Welcome\n\nClick to edit"),
                is_editing: false,
            }],
            next_id: 1,
            focus_handle: cx.focus_handle(),
        }
    }

    fn parse_markdown(content: &str) -> SharedString {
        let mut html = String::new();
        
        for line in content.lines() {
            let trimmed = line.trim();
            
            if trimmed.starts_with("# ") {
                html.push_str(&format!("📄 {}\n\n", &trimmed[2..]));
            } else if trimmed.starts_with("## ") {
                html.push_str(&format!("  📌 {}\n\n", &trimmed[3..]));
            } else if trimmed.starts_with("### ") {
                html.push_str(&format!("    • {}\n\n", &trimmed[4..]));
            } else if trimmed.starts_with("- ") {
                html.push_str(&format!("  • {}\n", &trimmed[2..]));
            } else if !trimmed.is_empty() {
                html.push_str(&format!("{}\n\n", trimmed));
            }
        }
        
        html.into()
    }

    fn add_block(&mut self, cx: &mut Context<Self>) {
        self.blocks.push(Block {
            id: self.next_id,
            content: "Type here...".into(),
            rendered: "Type here...".into(),
            is_editing: true,
        });
        self.next_id += 1;
        cx.notify();
    }

    fn toggle_edit(&mut self, block_id: usize, cx: &mut Context<Self>) {
        if let Some(block) = self.blocks.iter_mut().find(|b| b.id == block_id) {
            block.is_editing = !block.is_editing;
            if !block.is_editing {
                block.rendered = Self::parse_markdown(&block.content);
            }
        }
        cx.notify();
    }
}

impl Focusable for MarkdownWidget {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for MarkdownWidget {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        
        div()
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .bg(theme.background())
            .p_4()
            .gap_2()
            .overflow_hidden()
            .children(
                self.blocks.iter().map(|block| {
                    let block_id = block.id;
                    
                    if block.is_editing {
                        // Mode édition - affiche le contenu brut
                        div()
                            .p_3()
                            .min_h(px(100.0))
                            .bg(theme.surface)
                            .border_2()
                            .border_color(theme.accent)
                            .rounded_md()
                            .on_mouse_down(MouseButton::Left, cx.listener(move |this, _event, _window, cx| {
                                this.toggle_edit(block_id, cx);
                            }))
                            .child(
                                div()
                                    .text_color(theme.text)
                                    .font_family("monospace")
                                    .child(block.content.clone())
                            )
                            .child(
                                div()
                                    .mt_2()
                                    .text_sm()
                                    .text_color(theme.text_muted)
                                    .child("Click to preview")
                            )
                    } else {
                        // Mode preview - affiche le rendu
                        div()
                            .p_3()
                            .min_h(px(100.0))
                            .bg(theme.surface)
                            .border_1()
                            .border_color(theme.overlay)
                            .rounded_md()
                            .cursor_pointer()
                            .on_mouse_down(MouseButton::Left, cx.listener(move |this, _event, _window, cx| {
                                this.toggle_edit(block_id, cx);
                            }))
                            .child(
                                div()
                                    .text_color(theme.text)
                                    .child(block.rendered.clone())
                            )
                    }
                })
            )
            .child(
                // Bouton pour ajouter un bloc
                div()
                    .p_2()
                    .mt_2()
                    .bg(theme.overlay)
                    .rounded_md()
                    .cursor_pointer()
                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _event, _window, cx| {
                        this.add_block(cx);
                    }))
                    .child("+ Add block")
            )
    }
}
