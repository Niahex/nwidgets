mod modules;

pub use modules::{AudioModule, DateTimeModule, WorkspacesModule};

use gpui::*;

pub struct Panel;

impl Panel {
    pub fn new() -> Self {
        Self
    }
}

impl Render for Panel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let workspaces = cx.new(|cx| WorkspacesModule::new(cx));
        let datetime = cx.new(|cx| DateTimeModule::new(cx));
        let audio = cx.new(|cx| AudioModule::new(cx));

        div()
            .flex()
            .items_center()
            .justify_between()
            .h(px(40.))
            .w_full()
            .px_4()
            .bg(rgb(0x1e1e2e))
            .text_color(rgb(0xcdd6f4))
            // Left section
            .child(
                div()
                    .flex()
                    .gap_2()
                    .items_center()
                    .child("NWidgets")
            )
            // Center section
            .child(
                div()
                    .flex()
                    .gap_2()
                    .items_center()
                    .child(workspaces)
            )
            // Right section
            .child(
                div()
                    .flex()
                    .gap_3()
                    .items_center()
                    .child(audio)
                    .child(datetime)
            )
    }
}
