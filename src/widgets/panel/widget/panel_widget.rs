use crate::theme::ActiveTheme;
use crate::widgets::control_center::ControlCenterService;
use crate::widgets::panel::modules::{
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
        let theme = cx.theme();
        let corner_radius = px(18.);
        let panel_height = px(50.);

        div()
            .relative()
            .flex()
            .items_center()
            .justify_between()
            .h(panel_height + corner_radius)
            .w_full()
            // Panel principal (sans border)
            .child(
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .right_0()
                    .h(panel_height)
                    .px_3()
                    .bg(theme.bg)
                    .shadow_lg()
                    .text_color(theme.text)
                    .flex()
                    .items_center()
                    .justify_between()
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
                                    .id("control-center-trigger")
                                    .flex()
                                    .gap_0()
                                    .items_center()
                                    .h_full()
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
                                    ),
                            ),
                    ),
            )
            // Left corner avec bg
            .child(
                canvas(move |_, _, _| {}, {
                    let color = theme.bg;
                    move |_bounds, _, window, _| {
                        let ox = px(0.);
                        let oy = panel_height;
                        let mut path = PathBuilder::fill();
                        path.move_to(point(ox, oy + corner_radius));
                        path.arc_to(
                            point(corner_radius, corner_radius),
                            px(0.),
                            false,
                            true,
                            point(ox + corner_radius, oy),
                        );
                        path.line_to(point(ox, oy));
                        path.close();
                        if let Ok(built_path) = path.build() {
                            window.paint_path(built_path, color);
                        }
                    }
                })
                .absolute()
                .top(panel_height)
                .left_0()
                .size(corner_radius),
            )
            // Right corner avec bg
            .child(
                canvas(move |_, _, _| {}, {
                    let color = theme.bg;
                    move |bounds, _, window, _| {
                        let ox = bounds.origin.x;
                        let oy = panel_height;
                        let mut path = PathBuilder::fill();
                        path.move_to(point(ox, oy));
                        path.arc_to(
                            point(corner_radius, corner_radius),
                            px(0.),
                            false,
                            true,
                            point(ox + corner_radius, oy + corner_radius),
                        );
                        path.line_to(point(ox + corner_radius, oy));
                        path.close();
                        if let Ok(built_path) = path.build() {
                            window.paint_path(built_path, color);
                        }
                    }
                })
                .absolute()
                .top(panel_height)
                .right_0()
                .size(corner_radius),
            )
    }
}
