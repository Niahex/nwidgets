use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::widgets::*;
    use crate::theme::*;

    pub NwSlider = {{NwSlider}} {
        width: 200, height: 24

        draw_bg: {
            instance value: 0.5
            instance hover: 0.0

            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                let track_height = 4.0;
                let track_y = (self.rect_size.y - track_height) * 0.5;
                let radius = track_height * 0.5;

                sdf.box(0.0, track_y, self.rect_size.x, track_height, radius);
                sdf.fill(#4C566A);

                let filled_width = self.rect_size.x * self.value;
                sdf.box(0.0, track_y, filled_width, track_height, radius);
                sdf.fill(#88C0D0);

                let knob_radius = 8.0;
                let knob_x = filled_width;
                let knob_y = self.rect_size.y * 0.5;

                sdf.circle(knob_x, knob_y, knob_radius);
                sdf.fill(#ECEFF4);

                return sdf.result;
            }
        }

        animator: {
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
pub struct NwSlider {
    #[redraw]
    #[live]
    draw_bg: DrawQuad,

    #[walk]
    walk: Walk,

    #[layout]
    layout: Layout,

    #[animator]
    animator: Animator,

    #[live(0.5)]
    value: f32,

    #[live(0.0)]
    min: f32,

    #[live(1.0)]
    max: f32,

    #[live(0.01)]
    step: f32,

    #[rust]
    dragging: bool,
}

impl Widget for NwSlider {
    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        self.draw_bg.draw_walk(cx, walk);
        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        self.animator_handle_event(cx, event);

        match event.hits(cx, self.draw_bg.area()) {
            Hit::FingerDown(fe) => {
                self.dragging = true;
                self.update_value_from_pos(cx, fe.abs.x);
            }
            Hit::FingerMove(fe) => {
                if self.dragging {
                    self.update_value_from_pos(cx, fe.abs.x);
                }
            }
            Hit::FingerUp(_) => {
                self.dragging = false;
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

impl NwSlider {
    fn normalized_value(&self) -> f32 {
        (self.value - self.min) / (self.max - self.min)
    }

    fn update_value_from_pos(&mut self, cx: &mut Cx, abs_x: f64) {
        let rect = self.draw_bg.area().rect(cx);
        let rel_x = (abs_x - rect.pos.x).max(0.0).min(rect.size.x);
        let normalized = (rel_x / rect.size.x) as f32;
        let new_value = self.min + normalized * (self.max - self.min);
        let stepped = (new_value / self.step).round() * self.step;
        self.value = stepped.max(self.min).min(self.max);

        self.draw_bg.redraw(cx);

        cx.widget_action(
            self.widget_uid(),
            &HeapLiveIdPath::default(),
            NwSliderAction::Changed(self.value),
        );
    }
}

#[derive(Clone, Debug, DefaultNone)]
pub enum NwSliderAction {
    None,
    Changed(f32),
}
