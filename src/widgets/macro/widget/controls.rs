use crate::assets::Icon;
use crate::theme::Theme;
use gpui::prelude::*;
use gpui::*;

impl super::MacroWidget {
    pub(super) fn render_record_button(
        &self,
        theme: Theme,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let is_recording = self.macro_service.read(cx).is_recording();
        let icon_name = if is_recording {
            "recording-recording"
        } else {
            "recording-stopped"
        };

        div()
            .px_3()
            .py_2()
            .bg(theme.surface)
            .rounded(px(6.))
            .cursor_pointer()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _, _window, cx| {
                    if is_recording {
                        this.macro_service.update(cx, |service, cx| {
                            service.stop_recording(cx);
                        });
                    } else {
                        let name = format!("Macro {}", chrono::Local::now().format("%H:%M:%S"));
                        this.macro_service.update(cx, |service, cx| {
                            service.start_recording(name, cx);
                        });
                    }
                }),
            )
            .child(Icon::new(icon_name).size(px(20.)).color(theme.text))
    }

    pub(super) fn render_speed_control(
        &mut self,
        theme: Theme,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let speed = self.macro_service.read(cx).playback_speed();

        div()
            .px_3()
            .py_1()
            .bg(theme.surface)
            .border_1()
            .border_color(theme.border())
            .rounded(px(4.))
            .text_sm()
            .text_color(theme.text)
            .on_scroll_wheel(
                cx.listener(move |this, event: &ScrollWheelEvent, window, cx| {
                    let delta = event.delta.pixel_delta(window.line_height());
                    let delta_y: f32 = delta.y.into();
                    let change = if delta_y > 0.0 { 0.1 } else { -0.1 };
                    this.macro_service.update(cx, |service, cx| {
                        let new_speed = (service.playback_speed() + change).clamp(0.1, 10.0);
                        service.set_playback_speed(new_speed, cx);
                        this.speed_input = format!("{:.1}", new_speed);
                    });
                }),
            )
            .child(format!("{:.1}x", speed))
    }
}
