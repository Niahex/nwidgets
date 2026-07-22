use gpui::*;
use gpui_component::{h_flex, ActiveTheme, button::Button, init as init_components};
use gpui_platform::application;

fn main() {
    application().run(|cx: &mut App| {
        init_components(cx);

        let bounds = Bounds::centered(None, size(px(400.), px(300.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|_| HelloWorld)
            },
        )
        .unwrap();

        cx.activate(true);
    });
}

struct HelloWorld;

impl Render for HelloWorld {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .size_full()
            .bg(cx.theme().background)
            .items_center()
            .justify_center()
            .child(
                Button::new("hello-btn")
                    .label("Hello from nwidgets-core!")
                    .primary(),
            )
    }
}
