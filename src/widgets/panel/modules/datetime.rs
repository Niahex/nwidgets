use crate::theme::ActiveTheme;
use chrono::Timelike;
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
        let time = format!("{}", now.format("%H:%M")).into();
        let date = format!("{}", now.format("%a %d %b")).into();

        cx.spawn(move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            let mut cx = cx.clone();
            async move {
                // Calculate delay until next minute to sync with system clock
                let now = chrono::Local::now();
                let seconds_until_next_minute = 60 - now.second() as u64;
                cx.background_executor()
                    .timer(Duration::from_secs(seconds_until_next_minute))
                    .await;

                // Initial update after sync
                if let Ok(()) = this.update(&mut cx, |this, cx| {
                    let now = chrono::Local::now();
                    this.time = format!("{}", now.format("%H:%M")).into();
                    this.date = format!("{}", now.format("%a %d %b")).into();
                    cx.notify();
                }) {}

                // Loop every 60 seconds
                loop {
                    cx.background_executor()
                        .timer(Duration::from_secs(60))
                        .await;

                    let now = chrono::Local::now();
                    if let Ok(()) = this.update(&mut cx, |this, cx| {
                        this.time = format!("{}", now.format("%H:%M")).into();
                        this.date = format!("{}", now.format("%a %d %b")).into();
                        cx.notify();
                    }) {}
                }
            }
        })
        .detach();

        Self { time, date }
    }
}

impl Render for DateTimeModule {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let time = self.time.clone();
        let date = self.date.clone();

        div()
            .flex()
            .flex_col()
            .items_center()
            .px_2()
            .child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(time),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().text_muted)
                    .child(date),
            )
    }
}
