mod modules;

pub use modules::{
    ActiveWindowModule, BluetoothModule, DateTimeModule, MprisModule, NetworkModule,
    PomodoroModule, SinkModule, SourceModule, SystrayModule, WorkspacesModule,
};

use crate::services::control_center::ControlCenterService;
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
    control_center: Entity<ControlCenterService>,
}

impl Panel {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            active_window: cx.new(ActiveWindowModule::new),
            workspaces: cx.new(WorkspacesModule::new),
            pomodoro: cx.new(PomodoroModule::new),
            mpris: cx.new(MprisModule::new),
            systray: cx.new(SystrayModule::new),
            bluetooth: cx.new(BluetoothModule::new),
            network: cx.new(NetworkModule::new),
            sink: cx.new(SinkModule::new),
            source: cx.new(SourceModule::new),
            datetime: cx.new(DateTimeModule::new),
            control_center: ControlCenterService::global(cx),
        }
    }
}

impl Render for Panel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let bg_color = rgb(0x2e3440); // $polar0
        let text_color = rgb(0xeceff4); // $snow2
        let hover_bg = rgb(0x4c566a); // $polar3

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
                    // Group interactive modules for Control Center toggle
                    .child(
                        div()
                            .id("control-center-trigger")
                            .flex()
                            .gap_0()
                            .items_center()
                            .h_full()
                            .hover(|s| s.bg(hover_bg))
                            .rounded_md()
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _, _window, cx| {
                                this.control_center.update(cx, |cc, cx| {
                                    cc.toggle(cx);
                                });
                            }))
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
                            )
                    ),
            )
    }
}