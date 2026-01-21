use crate::services::ui::chat::ChatService;
use crate::services::system::hyprland::{ActiveWindowChanged, HyprlandService};
use crate::theme::ActiveTheme;
use crate::assets::Icon;
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
    // Cache
    cached_icon: SharedString,
    cached_class: SharedString,
    cached_title: SharedString,
}

impl ActiveWindowModule {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let hyprland = HyprlandService::global(cx);
        let chat = ChatService::global(cx);

        cx.subscribe(
            &hyprland,
            |this, hyprland, _event: &ActiveWindowChanged, cx| {
                this.update_cache(hyprland.read(cx).active_window().as_ref(), false);
                cx.notify();
            },
        )
        .detach();

        let active_window = hyprland.read(cx).active_window();
        let (icon, class, title) = Self::compute_window_info(active_window.as_ref(), false, 0);

        Self {
            hyprland,
            chat,
            site_index: 0,
            cached_icon: icon,
            cached_class: class,
            cached_title: title,
        }
    }

    fn compute_window_info(
        window: Option<&crate::services::system::hyprland::ActiveWindow>,
        chat_visible: bool,
        site_index: usize,
    ) -> (SharedString, SharedString, SharedString) {
        if chat_visible {
            let site_name = SITES[site_index].0;
            (
                site_name.to_lowercase().into(),
                "AI Chat".into(),
                site_name.into(),
            )
        } else if let Some(window) = window {
            (
                window.class.to_lowercase().into(),
                window.class.clone().into(),
                Self::extract_short_title(&window.title, 30).into(),
            )
        } else {
            ("nixos".into(), "NixOS".into(), "Nia".into())
        }
    }

    fn update_cache(
        &mut self,
        window: Option<&crate::services::system::hyprland::ActiveWindow>,
        chat_visible: bool,
    ) {
        let (icon, class, title) = Self::compute_window_info(window, chat_visible, self.site_index);
        self.cached_icon = icon;
        self.cached_class = class;
        self.cached_title = title;
    }

    fn get_icon_name(class: &str) -> String {
        class.to_lowercase()
    }

    fn extract_short_title(title: &str, max_chars: usize) -> String {
        let short_title = title
            .split(" - ")
            .next()
            .unwrap_or(title)
            .trim()
            .to_string();
        if short_title.chars().count() > max_chars {
            format!(
                "{}...",
                short_title.chars().take(max_chars - 3).collect::<String>()
            )
        } else {
            short_title
        }
    }

    fn format_class_name(class: &str) -> String {
        class.to_string()
    }

    pub fn next_site(&mut self) -> &'static str {
        self.site_index = (self.site_index + 1) % SITES.len();
        let site_name = SITES[self.site_index].0;
        self.cached_icon = site_name.to_lowercase().into();
        self.cached_class = "AI Chat".into();
        self.cached_title = site_name.into();
        SITES[self.site_index].1
    }

    pub fn current_site_name(&self) -> &'static str {
        SITES[self.site_index].0
    }
}

impl Render for ActiveWindowModule {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chat_visible = self.chat.read(cx).visible;
        let theme = cx.theme();

        // Update cache if chat visibility changed
        if chat_visible && self.cached_class != "AI Chat" {
            self.update_cache(None, true);
        }

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
                    this.chat
                        .update(cx, |chat, cx| chat.navigate(url.to_string(), cx));
                }
            }))
            .child(
                div().size(px(32.)).flex_shrink_0().child(
                    Icon::new(self.cached_icon.clone())
                        .size(px(32.))
                        .preserve_colors(true),
                ),
            )
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
                            .child(self.cached_class.clone()),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme.text)
                            .overflow_hidden()
                            .whitespace_nowrap()
                            .child(self.cached_title.clone()),
                    ),
            )
    }
}
