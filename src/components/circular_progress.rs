use gpui::prelude::*;
use gpui::*;
use crate::utils::Icon;

pub struct CircularProgress {
    size: Pixels,
    percent: u8,
    secondary_percent: Option<u8>,
    label: SharedString,
    color: Hsla,
    secondary_color: Hsla,
}

impl CircularProgress {
    pub fn new(size: Pixels) -> Self {
        Self {
            size,
            percent: 0,
            secondary_percent: None,
            label: "".into(),
            color: rgb(0x88c0d0).into(),
            secondary_color: rgb(0x8fbcbb).into(),
        }
    }

    pub fn percent(mut self, percent: u8) -> Self {
        self.percent = percent;
        self
    }

    pub fn secondary_percent(mut self, percent: u8) -> Self {
        self.secondary_percent = Some(percent);
        self
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    pub fn color(mut self, color: Hsla) -> Self {
        self.color = color;
        self
    }

    pub fn secondary_color(mut self, color: Hsla) -> Self {
        self.secondary_color = color;
        self
    }

    fn get_circle_icon(percent: u8) -> &'static str {
        if percent == 0 {
            "circle_0"
        } else if percent <= 20 {
            "circle_20"
        } else if percent <= 40 {
            "circle_40"
        } else if percent <= 60 {
            "circle_60"
        } else if percent <= 80 {
            "circle_80"
        } else {
            "circle_100"
        }
    }

    fn get_circle_color_icon(percent: u8) -> &'static str {
        if percent <= 20 {
            "circle_color_20"
        } else if percent <= 40 {
            "circle_color_40"
        } else if percent <= 60 {
            "circle_color_60"
        } else if percent <= 80 {
            "circle_color_80"
        } else {
            "circle_color_100"
        }
    }
}

impl IntoElement for CircularProgress {
    type Element = Div;

    fn into_element(self) -> Self::Element {
        div()
            .flex()
            .items_center()
            .gap_3()
            .child(
                div()
                    .size(self.size)
                    .relative()
                    .child(
                        div()
                            .size_full()
                            .child(
                                Icon::new(Self::get_circle_icon(self.percent))
                                    .size(self.size)
                                    .preserve_colors(true),
                            ),
                    )
                    .when(self.secondary_percent.is_some(), |this| {
                        let sec_percent = self.secondary_percent.unwrap();
                        let inner_size = self.size * 0.7;
                        let offset = (self.size - inner_size) / 2.0;
                        this.child(
                            div()
                                .absolute()
                                .top(offset)
                                .left(offset)
                                .size(inner_size)
                                .child(
                                    Icon::new(Self::get_circle_color_icon(sec_percent))
                                        .size(inner_size)
                                        .preserve_colors(true),
                                ),
                        )
                    })
                    .child(
                        div()
                            .absolute()
                            .inset_0()
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0xeceff4))
                            .child(self.label.clone()),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_0p5()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0xeceff4))
                            .child(format!("{}%", self.percent)),
                    )
                    .when(self.secondary_percent.is_some(), |this| {
                        this.child(
                            div()
                                .text_xs()
                                .text_color(rgb(0xd8dee9))
                                .child(format!("{}°C", self.secondary_percent.unwrap())),
                        )
                    }),
            )
    }
}
