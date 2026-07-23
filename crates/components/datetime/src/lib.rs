use chrono::Local;
use gpui::*;
use std::time::Duration;

pub struct DateTimeComponent {
    time: SharedString,
    date: SharedString,
}

impl DateTimeComponent {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let now = Local::now();
        let time: SharedString = now.format("%H:%M").to_string().into();
        let date: SharedString = now.format("%a %d %b").to_string().into();

        cx.spawn(async move |this, cx| loop {
            let now = Local::now();
            let seconds_until_next_minute = 60 - now.format("%S").to_string().parse::<u64>().unwrap_or(0);
            let sleep_duration = Duration::from_secs(seconds_until_next_minute.max(1));

            cx.background_executor().timer(sleep_duration).await;

            let now = Local::now();
            let new_time: SharedString = now.format("%H:%M").to_string().into();
            let new_date: SharedString = now.format("%a %d %b").to_string().into();

            if this
                .update(cx, |this, cx| {
                    if this.time != new_time || this.date != new_date {
                        this.time = new_time;
                        this.date = new_date;
                        cx.notify();
                    }
                })
                .is_err()
            {
                break;
            }
        })
        .detach();

        Self { time, date }
    }
}

use gpui_component::ActiveTheme;

impl Render for DateTimeComponent {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let text_muted = cx.theme().muted_foreground;
        let text_main = cx.theme().foreground;

        div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .px_2()
            .child(
                div()
                    .text_base()
                    .font_weight(FontWeight::BOLD)
                    .text_color(text_main)
                    .child(self.time.clone()),
            )
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(text_muted)
                    .child(self.date.clone()),
            )
    }
}
