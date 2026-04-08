use super::service::SystrayService;
use super::types::{TrayItem, TrayStateChanged};
use crate::assets::Icon;
use crate::theme::ActiveTheme;
use gpui::*;

pub struct SystrayWidget {
    systray: Entity<SystrayService>,
}

impl SystrayWidget {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let systray = SystrayService::global(cx);
        
        cx.subscribe(&systray, |_this, _systray, _event: &TrayStateChanged, cx| {
            cx.notify();
        })
        .detach();

        Self { systray }
    }

    fn render_tray_icon(&self, item: &TrayItem, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let item_clone = item.clone();
        
        let icon_name = if let Some(ref icon) = item.icon_name {
            Some(icon.clone())
        } else if item.title.to_lowercase().contains("steam") {
            Some("steam_tray".to_string())
        } else {
            None
        };
        
        div()
            .flex()
            .items_center()
            .justify_center()
            .w(px(32.))
            .h(px(32.))
            .rounded(px(6.))
            .hover(|style| style.bg(theme.systray_hover))
            .cursor_pointer()
            .on_mouse_down(MouseButton::Left, cx.listener(move |_this, _event, _window, cx| {
                let item = item_clone.clone();
                gpui_tokio::Tokio::spawn(cx, async move {
                    if let Err(e) = super::item_client::activate_item(
                        &item.service,
                        &item.object_path,
                        0,
                        0,
                    )
                    .await
                    {
                        log::error!("Failed to activate tray item: {}", e);
                    }
                })
                .detach();
            }))
            .child(
                if let Some(icon) = icon_name {
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(Icon::new(icon).size(px(20.)).color(theme.text))
                } else {
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_size(px(14.))
                        .text_color(theme.text)
                        .child(
                            item.title
                                .chars()
                                .next()
                                .unwrap_or('?')
                                .to_string()
                                .to_uppercase(),
                        )
                }
            )
    }
}

impl Render for SystrayWidget {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let items: Vec<TrayItem> = {
            let service = self.systray.read(cx);
            service.items.read().clone()
        };
        
        div()
            .flex()
            .gap_1()
            .items_center()
            .h_full()
            .children(items.iter().map(|item| self.render_tray_icon(item, cx)))
    }
}
