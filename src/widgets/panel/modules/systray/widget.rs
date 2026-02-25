use crate::widgets::panel::modules::systray::{SystrayChanged, SystrayService};
use crate::theme::ActiveTheme;
use gpui::prelude::*;
use gpui::*;

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
                let service_name = item.service_name.clone();
                let object_path = item.object_path.clone();
                
                div()
                    .id(("systray-item", idx))
                    .px_2()
                    .py_1()
                    .rounded_sm()
                    .hover(|style| style.bg(cx.theme().systray_hover))
                    .cursor_pointer()
                    .on_click(move |_event, _window, cx| {
                        let service = service_name.clone();
                        let path = object_path.clone();
                        gpui_tokio::Tokio::spawn(cx, async move {
                            SystrayService::activate_item(&service, &path).await;
                        })
                        .detach();
                    })
                    .child(
                        item.icon_name
                            .as_ref()
                            .map(|name| name.to_string())
                            .unwrap_or_else(|| item.title.to_string())
                    )
            }))
            .into_any_element()
    }
}
