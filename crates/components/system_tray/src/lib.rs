use gpui::*;
use gpui_component::Icon;
use nwidgets_service_system_tray::{SystemTrayService, SystemTrayStateChanged};
use std::path::PathBuf;

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
                let service_path_left = item.service_path.clone();
                let service_path_right = item.service_path.clone();
                let system_tray = self.system_tray.clone();
                let system_tray_right = self.system_tray.clone();

                let icon_element: AnyElement = if let Some(path) = item.icon_path {
                    img(PathBuf::from(path)).size(px(22.0)).into_any_element()
                } else {
                    Icon::new(SharedString::from(item.icon_name))
                        .size(px(22.0))
                        .text_color(text_main)
                        .into_any_element()
                };

                div()
                    .id(SharedString::from(format!("tray-item-{}", item.id)))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded_md()
                    .cursor_pointer()
                    .hover(|s| s.bg(rgb(0x3b4252)))
                    .on_mouse_down(MouseButton::Left, move |_event, _window, cx| {
                        system_tray.read(cx).activate_item(service_path_left.clone(), 0, 0);
                    })
                    .on_mouse_down(MouseButton::Right, move |_event, _window, cx| {
                        system_tray_right.read(cx).context_menu_item(service_path_right.clone(), 0, 0);
                    })
                    .child(icon_element)
            })
            .collect();

        div()
            .flex()
            .items_center()
            .gap_4()
            .children(icon_elements)
    }
}
