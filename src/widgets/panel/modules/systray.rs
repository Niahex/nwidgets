use gpui::prelude::*;
use gpui::*;
use crate::services::systray::{SystrayChanged, SystrayService};

pub struct SystrayModule {
    systray: Entity<SystrayService>,
}

impl SystrayModule {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let systray = SystrayService::global(cx);

        cx.subscribe(&systray, |_this, _systray, _event: &SystrayChanged, cx| {
            cx.notify();
        })
        .detach();

        Self { systray }
    }
}

impl Render for SystrayModule {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let items = self.systray.read(cx).items();
        let is_empty = items.is_empty();

        div()
            .flex()
            .gap_2()
            .items_center()
            .children(items.into_iter().enumerate().map(|(idx, item)| {
                div()
                    .id(("systray-item", idx))
                    .px_2()
                    .py_1()
                    .rounded_sm()
                    .hover(|style| style.bg(rgb(0x313244)))
                    .cursor_pointer()
                    .child(item.icon_name.unwrap_or_else(|| "📦".to_string()))
            }))
            .when(is_empty, |this| {
                this.child(
                    div()
                        .text_xs()
                        .text_color(rgb(0x6c7086))
                        .child("No tray items")
                )
            })
    }
}
