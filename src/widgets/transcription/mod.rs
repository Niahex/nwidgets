use crate::theme::*;
use gpui::*;

pub struct TranscriptionViewer {
    text: SharedString,
}

impl TranscriptionViewer {
    pub fn new(text: String) -> Self {
        Self {
            text: text.into(),
        }
    }

    pub fn append_text(&mut self, new_text: &str, cx: &mut Context<Self>) {
        if !self.text.is_empty() {
            self.text = format!("{} {}", self.text, new_text).into();
        } else {
            self.text = new_text.to_string().into();
        }
        cx.notify();
    }

    pub fn clear(&mut self, cx: &mut Context<Self>) {
        self.text = "".into();
        cx.notify();
    }

    fn on_copy(&mut self, _: &gpui::MouseDownEvent, window: &mut Window, _cx: &mut Context<Self>) {
        // Copy to clipboard
        _cx.write_to_clipboard(ClipboardItem::new_string(self.text.to_string()));
        println!("[TRANSCRIPTION] Text copied to clipboard");

        // Request to stop recording
        use crate::services::{TranscriptionEvent, TranscriptionEventService};
        TranscriptionEventService::send_event(TranscriptionEvent::StopRequested);

        // Close window
        window.remove_window();
    }

    fn on_cancel(&mut self, _: &gpui::MouseDownEvent, window: &mut Window, _cx: &mut Context<Self>) {
        println!("[TRANSCRIPTION] Cancelled");

        // Request to stop recording
        use crate::services::{TranscriptionEvent, TranscriptionEventService};
        TranscriptionEventService::send_event(TranscriptionEvent::StopRequested);

        // Close window
        window.remove_window();
    }
}

impl Render for TranscriptionViewer {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .w(px(600.))
            .min_h(px(200.))
            .max_h(px(400.))
            .bg(rgb(POLAR0))
            .border_2()
            .border_color(rgb(FROST1))
            .rounded_lg()
            .shadow_lg()
            .p_4()
            .flex()
            .flex_col()
            .gap_4()
            // Title
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::BOLD)
                    .text_color(rgb(SNOW0))
                    .child("Transcription")
            )
            // Transcription text area
            .child(
                div()
                    .flex_1()
                    .w_full()
                    .p_3()
                    .bg(rgb(POLAR1))
                    .border_1()
                    .border_color(rgb(POLAR3))
                    .rounded_md()
                    .text_sm()
                    .text_color(rgb(SNOW1))
                    .child(if self.text.is_empty() {
                        div()
                            .text_color(rgb(POLAR3))
                            .italic()
                            .child("En attente de la transcription...")
                    } else {
                        div().child(self.text.clone())
                    })
            )
            // Buttons
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_3()
                    .justify_end()
                    .child(
                        div()
                            .px_4()
                            .py_2()
                            .bg(rgb(POLAR2))
                            .border_1()
                            .border_color(rgb(POLAR3))
                            .rounded_md()
                            .text_sm()
                            .text_color(rgb(SNOW0))
                            .cursor_pointer()
                            .hover(|style| style.bg(rgb(POLAR3)))
                            .on_mouse_down(gpui::MouseButton::Left, cx.listener(Self::on_cancel))
                            .child("Cancel")
                    )
                    .child(
                        div()
                            .px_4()
                            .py_2()
                            .bg(rgb(FROST1))
                            .border_1()
                            .border_color(rgb(FROST2))
                            .rounded_md()
                            .text_sm()
                            .text_color(rgb(POLAR0))
                            .cursor_pointer()
                            .hover(|style| style.bg(rgb(FROST2)))
                            .on_mouse_down(gpui::MouseButton::Left, cx.listener(Self::on_copy))
                            .child("Copy")
                    )
            )
    }
}
