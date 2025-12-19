use gpui::prelude::*;
use gpui::*;
use std::time::Duration;

pub struct DateTimeModule {
    time: SharedString,
    date: SharedString,
}

impl DateTimeModule {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let now = chrono::Local::now();
        let time = now.format("%H:%M").to_string().into();
        let date = now.format("%a %d %b").to_string().into();

        // Update every 60 seconds
        cx.spawn(async move |this, cx| loop {
            cx.background_executor()
                .timer(Duration::from_secs(60))
                .await;

            let now = chrono::Local::now();
            if let Ok(()) = this.update(cx, |this, cx| {
                this.time = now.format("%H:%M").to_string().into();
                this.date = now.format("%a %d %b").to_string().into();
                cx.notify();
            }) {}
        })
        .detach();

        Self { time, date }
    }
}

impl Render for DateTimeModule {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .items_center()
            .px_2()
            .child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(self.time.clone()),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(0xd8dee9)) // $snow0
                    .child(self.date.clone()),
            )
    }
}
