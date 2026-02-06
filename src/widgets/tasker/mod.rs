use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::widgets::*;
    use makepad_draw::shader::std::*;
    use crate::theme::*;

    pub Tasker = {{Tasker}} {
        width: 800, height: 600

        show_bg: true
        draw_bg: {
            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, 12.0);
                sdf.fill((NORD_POLAR_0));
                sdf.stroke((NORD_FROST_1), 1.0);
                return sdf.result;
            }
        }

        flow: Down
        padding: 16
        spacing: 12

        visible: false

        header = <View> {
            width: Fill, height: Fit
            flow: Right
            align: {x: 0.0, y: 0.5}
            spacing: 12

            title = <Label> {
                draw_text: { 
                    text_style: <THEME_FONT_BOLD> { font_size: 18.0 }, 
                    color: (THEME_COLOR_TEXT_DEFAULT) 
                }
                text: "Tasker"
            }
        }

        content = <View> {
            width: Fill, height: Fill
            flow: Down
            spacing: 8

            project_list = <View> {
                width: Fill, height: Fill
                flow: Down
                spacing: 4
            }
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct Tasker {
    #[deref]
    view: View,
}

impl Widget for Tasker {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if let Event::KeyDown(ke) = event {
            if ke.key_code == KeyCode::Escape {
                self.hide(cx);
                return;
            }
        }

        self.view.handle_event(cx, event, scope);
    }
}

impl Tasker {
    pub fn show(&mut self, cx: &mut Cx) {
        self.view.apply_over(cx, live! { visible: true });
        
        cx.widget_action(self.view.widget_uid(), &Scope::empty().path, TaskerAction::Shown);
        cx.new_next_frame();
        self.view.redraw(cx);
    }

    pub fn hide(&mut self, cx: &mut Cx) {
        self.view.apply_over(cx, live! { visible: false });
        
        cx.widget_action(self.view.widget_uid(), &Scope::empty().path, TaskerAction::Hidden);
        self.view.redraw(cx);
    }
}

#[derive(Clone, Debug, DefaultNone)]
pub enum TaskerAction {
    None,
    Close,
    Shown,
    Hidden,
    ProjectSelected(String),
    TaskToggled(String),
}
