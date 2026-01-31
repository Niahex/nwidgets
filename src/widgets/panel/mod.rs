use makepad_widgets::*;

pub mod modules;

pub use modules::*;

live_design! {
    use link::theme::*;
    use link::widgets::*;
    use crate::theme::*;
    use crate::widgets::panel::modules::*;

    pub Panel = {{Panel}} {
        width: Fill, height: 68

        show_bg: true
        draw_bg: {
            color: (NORD_POLAR_0)
        }

        flow: Right
        align: {x: 0.0, y: 0.5}
        padding: {left: 12, right: 12, top: 0, bottom: 0}

        left_section = <View> {
            width: Fit, height: Fill
            flow: Right
            align: {x: 0.0, y: 0.5}
            spacing: 8

            active_window = <ActiveWindowModule> {}
        }

        center_section = <View> {
            width: Fill, height: Fill
            flow: Right
            align: {x: 0.5, y: 0.5}
            spacing: 12

            pomodoro = <PomodoroModule> {}
            workspaces = <WorkspacesModule> {}
            mpris = <MprisModule> {}
        }

        right_section = <View> {
            width: Fit, height: Fill
            flow: Right
            align: {x: 1.0, y: 0.5}
            spacing: 0

            systray = <SystrayModule> {}

            control_trigger = <View> {
                width: Fit, height: Fill
                flow: Right
                align: {x: 0.5, y: 0.5}
                spacing: 0
                cursor: Hand

                bluetooth = <BluetoothModule> {}
                network = <NetworkModule> {}
                source = <SourceModule> {}
                sink = <SinkModule> {}
                datetime = <DateTimeModule> {}
            }
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct Panel {
    #[deref]
    view: View,
}

impl Widget for Panel {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }
}
