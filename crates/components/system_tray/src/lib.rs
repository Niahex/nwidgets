use gpui::*;
use gpui_component::Icon;
use nwidgets_service_system_tray::{SystemTrayService, SystemTrayStateChanged};

pub struct SystemTrayComponent {
    system_tray: Entity<SystemTrayService>,
}

impl SystemTrayComponent {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let system_tray = SystemTrayService::global(cx);
        cx.subscribe(&system_tray, |_, _, _: &SystemTrayStateChanged, cx| {
            cx.notify();
        })
        .detach();

        Self { system_tray }
    }
}

impl Render for SystemTrayComponent {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let text_main = rgb(0xd8dee9);
        let items = self.system_tray.read(cx).state.items.clone();

        let icon_elements: Vec<_> = items
            .into_iter()
            .map(|item| {
                div()
                    .id(SharedString::from(format!("tray-item-{}", item.id)))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded_md()
                    .cursor_pointer()
                    .hover(|s| s.bg(rgb(0x3b4252)))
                    .child(
                        Icon::new(SharedString::from(item.icon_name))
                            .size(px(22.0))
                            .text_color(text_main),
                    )
            })
            .collect();

        div()
            .flex()
            .items_center()
            .gap_4()
            .children(icon_elements)
    }
}
