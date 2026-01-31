use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::widgets::*;
    use crate::theme::*;

    WorkspaceIndicator = <View> {
        width: 8, height: 8

        show_bg: true
        draw_bg: {
            instance active: 0.0
            instance occupied: 0.0

            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                let center = self.rect_size * 0.5;
                let radius = min(center.x, center.y);

                let color = mix(
                    mix(#4C566A, #88C0D0, self.occupied),
                    #88C0D0,
                    self.active
                );

                sdf.circle(center.x, center.y, radius);
                sdf.fill(color);

                return sdf.result;
            }
        }
    }

    pub WorkspacesModule = {{WorkspacesModule}} {
        width: Fit, height: Fill
        flow: Right
        align: {x: 0.5, y: 0.5}
        spacing: 6
        padding: {left: 8, right: 8}

        ws1 = <WorkspaceIndicator> {}
        ws2 = <WorkspaceIndicator> {}
        ws3 = <WorkspaceIndicator> {}
        ws4 = <WorkspaceIndicator> {}
        ws5 = <WorkspaceIndicator> {}
        ws6 = <WorkspaceIndicator> {}
        ws7 = <WorkspaceIndicator> {}
        ws8 = <WorkspaceIndicator> {}
        ws9 = <WorkspaceIndicator> {}
        ws10 = <WorkspaceIndicator> {}
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct WorkspacesModule {
    #[deref]
    view: View,

    #[rust]
    active_workspace: usize,

    #[rust]
    occupied_workspaces: Vec<usize>,
}

impl Widget for WorkspacesModule {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }
}

impl WorkspacesModule {
    pub fn set_active(&mut self, _cx: &mut Cx, workspace: usize) {
        self.active_workspace = workspace;
    }

    pub fn set_occupied(&mut self, _cx: &mut Cx, workspaces: Vec<usize>) {
        self.occupied_workspaces = workspaces;
    }
}
