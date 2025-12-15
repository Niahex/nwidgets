mod modules;

pub use modules::{
    AudioModule, DateTimeModule, MprisModule, NetworkModule,
    PomodoroModule, SystrayModule, WorkspacesModule
};

use gpui::*;

pub struct Panel;

impl Panel {
    pub fn new() -> Self {
        Self
    }
}

impl Render for Panel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Create all modules
        let workspaces = cx.new(|cx| WorkspacesModule::new(cx));
        let pomodoro = cx.new(|cx| PomodoroModule::new(cx));
        let mpris = cx.new(|cx| MprisModule::new(cx));
        let systray = cx.new(|cx| SystrayModule::new(cx));
        let network = cx.new(|cx| NetworkModule::new(cx));
        let audio = cx.new(|cx| AudioModule::new(cx));
        let datetime = cx.new(|cx| DateTimeModule::new(cx));

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
                    .gap_3()
                    .items_center()
                    .child(
                        div()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0x89b4fa))
                            .child("NWidgets")
                    )
            )
            // Center section
            .child(
                div()
                    .flex()
                    .gap_4()
                    .items_center()
                    .child(pomodoro)
                    .child(workspaces)
                    .child(mpris)
            )
            // Right section
            .child(
                div()
                    .flex()
                    .gap_3()
                    .items_center()
                    .child(systray)
                    .child(network)
                    .child(audio)
                    .child(datetime)
            )
    }
}
