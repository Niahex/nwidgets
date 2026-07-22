use std::path::PathBuf;
use gpui::prelude::FluentBuilder;
use gpui::*;
use nwidgets_service_niri::{ActiveWindowChanged, NiriActiveWindowService};

pub struct ActiveWindowComponent {
    app_id: SharedString,
    class: SharedString,
    title: SharedString,
}

impl ActiveWindowComponent {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let niri = NiriActiveWindowService::global(cx);
        let active = niri.read(cx).active_window.clone();

        let (app_id, class, title) = Self::compute_window_info(&active.app_id, &active.title);

        cx.subscribe(&niri, |this, _service, event: &ActiveWindowChanged, cx| {
            let (app_id, class, title) = Self::compute_window_info(&event.0.app_id, &event.0.title);
            this.app_id = app_id;
            this.class = class;
            this.title = title;
            cx.notify();
        })
        .detach();

        Self {
            app_id,
            class,
            title,
        }
    }

    fn compute_window_info(app_id: &str, title: &str) -> (SharedString, SharedString, SharedString) {
        if app_id.is_empty() && title.is_empty() {
            ("nixos".into(), "NixOS".into(), "Nia".into())
        } else {
            let class_str = if app_id.is_empty() {
                "Desktop"
            } else {
                app_id
            };
            (
                app_id.to_string().into(),
                class_str.to_string().into(),
                Self::extract_short_title(title, 35).into(),
            )
        }
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

    fn resolve_icon_path(&self) -> PathBuf {
        let assets_base = std::env::var("NWIDGETS_ASSETS_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("assets"));

        let candidates = [
            self.app_id.to_lowercase(),
            self.app_id.split('.').last().unwrap_or("").to_lowercase(),
            self.class.to_lowercase(),
        ];

        for name in candidates {
            if name.is_empty() {
                continue;
            }
            let svg = assets_base.join(format!("{}.svg", name));
            if svg.exists() {
                return svg;
            }
            let png = assets_base.join(format!("{}.png", name));
            if png.exists() {
                return png;
            }
        }

        let fallback = assets_base.join("none.svg");
        if fallback.exists() {
            fallback
        } else {
            assets_base.join("nixos.svg")
        }
    }
}

impl Render for ActiveWindowComponent {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let text_muted = rgb(0x4c566a);
        let text_main = rgb(0xe5e9f0);
        let icon_path = self.resolve_icon_path();

        div()
            .id("active-window-component")
            .flex()
            .gap_2()
            .items_center()
            .px_3()
            .py_1()
            .min_w(px(200.0))
            .max_w(px(450.0))
            .child(
                div()
                    .size(px(28.0))
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(img(icon_path).size(px(28.0))),
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
                            .text_color(text_muted)
                            .overflow_hidden()
                            .whitespace_nowrap()
                            .child(self.class.clone()),
                    )
                    .when(!self.title.is_empty(), |this| {
                        this.child(
                            div()
                                .text_sm()
                                .text_color(text_main)
                                .overflow_hidden()
                                .whitespace_nowrap()
                                .child(self.title.clone()),
                        )
                    }),
            )
    }
}
