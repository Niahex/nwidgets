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

        if items.is_empty() {
            return div().into_any_element();
        }

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
            .into_any_element()
    }
}
