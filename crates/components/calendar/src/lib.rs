use gpui::*;
use gpui_component::calendar::{Calendar, CalendarState};

pub struct CalendarComponent {
    calendar_state: Entity<CalendarState>,
}

impl CalendarComponent {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let calendar_state = cx.new(|cx| CalendarState::new(window, cx));
        Self { calendar_state }
    }
}

impl Render for CalendarComponent {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let card_bg = rgb(0x3b4252);

        div()
            .w_full()
            .flex()
            .flex_col()
            .p_3()
            .bg(card_bg)
            .rounded_xl()
            .child(Calendar::new(&self.calendar_state))
    }
}
