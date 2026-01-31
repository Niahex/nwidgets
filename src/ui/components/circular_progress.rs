use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::widgets::*;
    use crate::theme::*;

    pub NwCircularProgress = {{NwCircularProgress}} {
        width: 60, height: 60

        draw_bg: {
            instance value: 0.5
            instance secondary_value: 0.0

            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                let center = self.rect_size * 0.5;
                let radius = min(center.x, center.y) - 4.0;
                let thickness = 4.0;

                sdf.circle(center.x, center.y, radius);
                sdf.stroke(#4C566A, thickness);

                let angle = self.value * 6.283185;
                let start_angle = -1.5707963;
                let end_angle = start_angle + angle;

                sdf.move_to(
                    center.x + radius * cos(start_angle),
                    center.y + radius * sin(start_angle)
                );
                sdf.arc(center.x, center.y, radius, start_angle, end_angle);
                sdf.stroke(#88C0D0, thickness);

                return sdf.result;
            }
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct NwCircularProgress {
    #[redraw]
    #[live]
    draw_bg: DrawQuad,

    #[walk]
    walk: Walk,

    #[layout]
    layout: Layout,

    #[live(0.0)]
    value: f32,

    #[live(0.0)]
    secondary_value: f32,
}

impl Widget for NwCircularProgress {
    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        self.draw_bg.draw_walk(cx, walk);
        DrawStep::done()
    }

    fn handle_event(&mut self, _cx: &mut Cx, _event: &Event, _scope: &mut Scope) {
    }
}

impl NwCircularProgress {
    pub fn set_value(&mut self, cx: &mut Cx, value: f32) {
        self.value = value.max(0.0).min(1.0);
        self.draw_bg.redraw(cx);
    }

    pub fn set_secondary_value(&mut self, cx: &mut Cx, value: f32) {
        self.secondary_value = value.max(0.0).min(1.0);
        self.draw_bg.redraw(cx);
    }
}
