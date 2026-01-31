use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::widgets::*;
    use crate::theme::*;

    pub NwToggle = {{NwToggle}} {
        width: 44, height: 24

        draw_bg: {
            instance active: 0.0
            instance hover: 0.0

            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                let radius = self.rect_size.y * 0.5;

                let bg_color = mix(
                    #4C566A,
                    #88C0D0,
                    self.active
                );

                sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, radius);
                sdf.fill(bg_color);

                let knob_radius = radius - 3.0;
                let knob_x = mix(
                    radius,
                    self.rect_size.x - radius,
                    self.active
                );

                sdf.circle(knob_x, radius, knob_radius);
                sdf.fill(#ECEFF4);

                return sdf.result;
            }
        }

        animator: {
            active = {
                default: off
                off = {
                    from: {all: Forward {duration: 0.2}}
                    apply: {draw_bg: {active: 0.0}}
                }
                on = {
                    from: {all: Forward {duration: 0.2}}
                    apply: {draw_bg: {active: 1.0}}
                }
            }
            hover = {
                default: off
                off = {
                    from: {all: Forward {duration: 0.1}}
                    apply: {draw_bg: {hover: 0.0}}
                }
                on = {
                    from: {all: Snap}
                    apply: {draw_bg: {hover: 1.0}}
                }
            }
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct NwToggle {
    #[redraw]
    #[live]
    draw_bg: DrawQuad,

    #[walk]
    walk: Walk,

    #[layout]
    layout: Layout,

    #[animator]
    animator: Animator,

    #[rust]
    is_active: bool,

    #[rust]
    area: Area,
}

impl Widget for NwToggle {
    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        self.draw_bg.draw_walk(cx, walk);
        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        self.animator_handle_event(cx, event);

        match event.hits(cx, self.draw_bg.area()) {
            Hit::FingerDown(_) => {
                self.is_active = !self.is_active;
                if self.is_active {
                    self.animator_play(cx, ids!(active.on));
                } else {
                    self.animator_play(cx, ids!(active.off));
                }
                cx.widget_action(
                    self.widget_uid(),
                    &HeapLiveIdPath::default(),
                    NwToggleAction::Changed(self.is_active),
                );
            }
            Hit::FingerHoverIn(_) => {
                self.animator_play(cx, ids!(hover.on));
            }
            Hit::FingerHoverOut(_) => {
                self.animator_play(cx, ids!(hover.off));
            }
            _ => {}
        }
    }
}

#[derive(Clone, Debug, DefaultNone)]
pub enum NwToggleAction {
    None,
    Changed(bool),
}
