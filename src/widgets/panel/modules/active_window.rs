use crate::services::chat::ChatService;
use crate::services::hyprland::{ActiveWindowChanged, HyprlandService};
use crate::utils::Icon;
use gpui::prelude::*;
use gpui::*;

const SITES: &[(&str, &str)] = &[
    ("Gemini", "https://gemini.google.com/app"),
    ("Perplexity", "https://www.perplexity.ai/"),
    ("Prime", "https://chat.primeintellect.ai/"),
    ("AI Studio", "https://aistudio.google.com/live"),
];

pub struct ActiveWindowModule {
    hyprland: Entity<HyprlandService>,
    chat: Entity<ChatService>,
    site_index: usize,
}

impl ActiveWindowModule {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let hyprland = HyprlandService::global(cx);
        let chat = ChatService::global(cx);

        cx.subscribe(&hyprland, |_this, _hyprland, _event: &ActiveWindowChanged, cx| {
            cx.notify();
        }).detach();

        Self { hyprland, chat, site_index: 0 }
    }

    fn get_icon_name(class: &str) -> String {
        class.to_lowercase()
    }

    fn extract_short_title(title: &str, max_chars: usize) -> String {
        let short_title = title.split(" - ").next().unwrap_or(title).trim().to_string();
        if short_title.chars().count() > max_chars {
            format!("{}...", short_title.chars().take(max_chars - 3).collect::<String>())
        } else {
            short_title
        }
    }

    fn format_class_name(class: &str) -> String {
        class.to_string()
    }

    pub fn next_site(&mut self) -> &'static str {
        self.site_index = (self.site_index + 1) % SITES.len();
        SITES[self.site_index].1
    }

    pub fn current_site_name(&self) -> &'static str {
        SITES[self.site_index].0
    }
}

impl Render for ActiveWindowModule {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let active_window = self.hyprland.read(cx).active_window();
        let chat_visible = self.chat.read(cx).visible;
        let theme = cx.global::<crate::theme::Theme>();

        let (icon_name, class_text, title_text) = if chat_visible {
            let site_name = self.current_site_name();
            (site_name.to_lowercase(), "AI Chat".to_string(), site_name.to_string())
        } else if let Some(window) = active_window {
            (
                Self::get_icon_name(&window.class),
                Self::format_class_name(&window.class),
                Self::extract_short_title(&window.title, 30),
            )
        } else {
            ("nixos".to_string(), "NixOS".to_string(), "Nia".to_string())
        };

        div()
            .id("active-window-module")
            .flex()
            .gap_2()
            .items_center()
            .px_3()
            .py_2()
            .min_w(px(350.))
            .max_w(px(450.))
            .cursor_pointer()
            .on_click(cx.listener(|this, _, _window, cx| {
                if this.chat.read(cx).visible {
                    let url = this.next_site();
                    this.chat.update(cx, |chat, cx| chat.navigate(url.to_string(), cx));
                }
            }))
            .child(div().size(px(32.)).flex_shrink_0().child(
                Icon::new(icon_name).size(px(32.)).preserve_colors(true),
            ))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_0p5()
                    .flex_1()
                    .min_w_0()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(rgba(0xd8dee966))
                            .overflow_hidden()
                            .whitespace_nowrap()
                            .child(class_text),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme.text)
                            .overflow_hidden()
                            .whitespace_nowrap()
                            .child(title_text),
                    ),
            )
    }
}
