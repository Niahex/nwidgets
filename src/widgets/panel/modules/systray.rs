use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::widgets::*;
    use crate::theme::*;

    pub SystrayModule = {{SystrayModule}} {
        width: Fit, height: Fill
        flow: Right
        align: {x: 0.5, y: 0.5}
        spacing: 4
        padding: {left: 4, right: 4}
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct SystrayModule {
    #[deref]
    view: View,
}

impl Widget for SystrayModule {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }
}
