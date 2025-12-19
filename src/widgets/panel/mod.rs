mod modules;

pub use modules::{
    ActiveWindowModule, BluetoothModule, DateTimeModule, MprisModule, NetworkModule,
    PomodoroModule, SinkModule, SourceModule, SystrayModule, WorkspacesModule,
};

use gpui::*;

pub struct Panel {
    active_window: Entity<ActiveWindowModule>,
    workspaces: Entity<WorkspacesModule>,
    pomodoro: Entity<PomodoroModule>,
    mpris: Entity<MprisModule>,
    systray: Entity<SystrayModule>,
    bluetooth: Entity<BluetoothModule>,
    network: Entity<NetworkModule>,
    sink: Entity<SinkModule>,
    source: Entity<SourceModule>,
    datetime: Entity<DateTimeModule>,
}

impl Panel {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            active_window: cx.new(|cx| ActiveWindowModule::new(cx)),
            workspaces: cx.new(|cx| WorkspacesModule::new(cx)),
            pomodoro: cx.new(|cx| PomodoroModule::new(cx)),
            mpris: cx.new(|cx| MprisModule::new(cx)),
            systray: cx.new(|cx| SystrayModule::new(cx)),
            bluetooth: cx.new(|cx| BluetoothModule::new(cx)),
            network: cx.new(|cx| NetworkModule::new(cx)),
            sink: cx.new(|cx| SinkModule::new(cx)),
            source: cx.new(|cx| SourceModule::new(cx)),
            datetime: cx.new(|cx| DateTimeModule::new(cx)),
        }
    }
}

impl Render for Panel {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let bg_color = rgb(0x2e3440); // $polar0
        let text_color = rgb(0xeceff4); // $snow2

        div()
            .flex()
            .items_center()
            .justify_between()
            .h(px(50.))
            .w_full()
            .px_3()
            .bg(bg_color)
            .text_color(text_color)
            // Left section - Active window info
            .child(
                div()
                    .flex()
                    .gap_2()
                    .items_center()
                    .h_full()
                    .child(self.active_window.clone()),
            )
            // Center section - takes remaining space
            .child(
                div()
                    .flex()
                    .flex_1()
                    .gap_2()
                    .items_center()
                    .justify_center()
                    .h_full()
                    .child(self.pomodoro.clone())
                    .child(self.workspaces.clone())
                    .child(self.mpris.clone()),
            )
            // Right section
            .child(
                div()
                    .flex()
                    .gap_0()
                    .items_center()
                    .h_full()
                    .child(div().flex().items_center().child(self.systray.clone()))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .w(px(32.))
                            .h(px(32.))
                            .child(self.bluetooth.clone()),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .w(px(32.))
                            .h(px(32.))
                            .child(self.network.clone()),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .w(px(32.))
                            .h(px(32.))
                            .child(self.source.clone()),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .w(px(32.))
                            .h(px(32.))
                            .child(self.sink.clone()),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .px_3()
                            .child(self.datetime.clone()),
                    ),
            )
    }
}
