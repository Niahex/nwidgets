use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::widgets::*;
    use crate::theme::*;

    NotificationCard = <View> {
        width: Fill, height: Fit
        flow: Down
        padding: 12
        spacing: 8

        show_bg: true
        draw_bg: {
            color: (NORD_POLAR_1)
            radius: 8.0
        }

        header = <View> {
            width: Fill, height: Fit
            flow: Right
            align: {x: 0.0, y: 0.5}
            spacing: 8

            app_icon = <View> {
                width: 24, height: 24
            }

            app_name = <Label> {
                draw_text: { text_style: <THEME_FONT_REGULAR> { font_size: 11.0 }, color: (THEME_COLOR_TEXT_MUTE) }
                text: "App"
            }

            time = <Label> {
                draw_text: { text_style: <THEME_FONT_REGULAR> { font_size: 10.0 }, color: (THEME_COLOR_TEXT_MUTE) }
                text: "now"
            }
        }

        summary = <Label> {
            draw_text: { text_style: <THEME_FONT_BOLD> { font_size: 12.0 }, color: (THEME_COLOR_TEXT_DEFAULT) }
            text: "Notification Title"
        }

        body = <Label> {
            draw_text: { text_style: <THEME_FONT_REGULAR> { font_size: 11.0 }, color: (THEME_COLOR_TEXT_MUTE) }
            text: "Notification body text"
        }
    }

    pub Notifications = {{Notifications}} {
        width: 380, height: Fit
        flow: Down
        spacing: 8
        padding: 8

        visible: false
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct Notifications {
    #[deref]
    view: View,

    #[rust]
    notifications: Vec<Notification>,
}

#[derive(Clone, Debug)]
pub struct Notification {
    pub id: u32,
    pub app_name: String,
    pub summary: String,
    pub body: String,
    pub timestamp: i64,
}

impl Widget for Notifications {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }
}

impl Notifications {
    pub fn add_notification(&mut self, cx: &mut Cx, notification: Notification) {
        self.notifications.insert(0, notification);
        self.view.redraw(cx);
    }

    pub fn remove_notification(&mut self, cx: &mut Cx, id: u32) {
        self.notifications.retain(|n| n.id != id);
        self.view.redraw(cx);
    }

    pub fn clear_all(&mut self, cx: &mut Cx) {
        self.notifications.clear();
        self.view.redraw(cx);
    }
}


