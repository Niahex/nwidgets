use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::widgets::*;

    pub List = {{List}} {
        width: Fill, height: Fill
        flow: Down
        scroll_bars: <ScrollBars> {}
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct List {
    #[deref]
    pub view: View,
}

impl Widget for List {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }
}

impl List {
    pub fn item(&mut self, _cx: &mut Cx, entry_id: LiveId, _template: LiveId) -> WidgetRef {
        self.view.widget(&[entry_id])
    }
    
    pub fn widget(&mut self, entry_id: LiveId) -> WidgetRef {
        self.view.widget(&[entry_id])
    }
}
