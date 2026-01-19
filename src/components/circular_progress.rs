use gpui::prelude::*;
use gpui::*;
use std::f32::consts::PI;

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
}

impl IntoElement for CircularProgress {
    type Element = Div;

    fn into_element(self) -> Self::Element {
        let size = self.size;
        let percent = self.percent;
        let temp = self.secondary_percent.unwrap_or(0);
        let color = self.color;
        let temp_color = self.secondary_color;
        let label = self.label.clone();

        div()
            .flex()
            .items_center()
            .gap_3()
            .child(
                div()
                    .size(size)
                    .relative()
                    .child(
                        canvas(
                            move |bounds, _window, _cx| (bounds, percent, temp, color, temp_color),
                            move |bounds, (_, percent, temp, color, temp_color), window, _cx| {
                                let center = bounds.center();
                                let stroke_width = size * 0.08;
                                let radius = (size - stroke_width) / 2.0;
                                
                                // Arc 1: Température (45° à 220°) - 175° d'espace
                                let arc1_start = 45.0 * PI / 180.0;
                                let arc1_end = 220.0 * PI / 180.0;
                                let arc1_sweep = arc1_end - arc1_start;
                                
                                // Background arc1
                                let bg1_start_pt = center + point(radius * arc1_start.cos(), radius * arc1_start.sin());
                                let bg1_end_pt = center + point(radius * arc1_end.cos(), radius * arc1_end.sin());
                                let mut bg1 = PathBuilder::stroke(stroke_width);
                                bg1.move_to(bg1_start_pt);
                                bg1.arc_to(point(radius, radius), px(0.0), arc1_sweep > PI, true, bg1_end_pt);
                                if let Ok(path) = bg1.build() {
                                    window.paint_path(path, rgb(0x4c566a));
                                }
                                
                                // Foreground arc1 (température)
                                if temp > 0 {
                                    let value1 = (temp as f32 / 100.0).max(1.0 / 360.0);
                                    let fg1_end_angle = arc1_start + arc1_sweep * value1;
                                    let fg1_end_pt = center + point(radius * fg1_end_angle.cos(), radius * fg1_end_angle.sin());
                                    
                                    let mut fg1 = PathBuilder::stroke(stroke_width);
                                    fg1.move_to(bg1_start_pt);
                                    fg1.arc_to(point(radius, radius), px(0.0), arc1_sweep * value1 > PI, true, fg1_end_pt);
                                    if let Ok(path) = fg1.build() {
                                        window.paint_path(path, temp_color);
                                    }
                                }
                                
                                // Arc 2: Usage (230° à 360°) - 130° d'espace
                                let arc2_start = 230.0 * PI / 180.0;
                                let arc2_end = 360.0 * PI / 180.0;
                                let arc2_sweep = arc2_end - arc2_start;
                                
                                // Background arc2
                                let bg2_start_pt = center + point(radius * arc2_start.cos(), radius * arc2_start.sin());
                                let bg2_end_pt = center + point(radius * arc2_end.cos(), radius * arc2_end.sin());
                                let mut bg2 = PathBuilder::stroke(stroke_width);
                                bg2.move_to(bg2_start_pt);
                                bg2.arc_to(point(radius, radius), px(0.0), arc2_sweep > PI, true, bg2_end_pt);
                                if let Ok(path) = bg2.build() {
                                    window.paint_path(path, rgb(0x4c566a));
                                }
                                
                                // Foreground arc2 (usage)
                                if percent > 0 {
                                    let value2 = (percent as f32 / 100.0).max(1.0 / 360.0);
                                    let fg2_end_angle = arc2_start + arc2_sweep * value2;
                                    let fg2_end_pt = center + point(radius * fg2_end_angle.cos(), radius * fg2_end_angle.sin());
                                    
                                    let mut fg2 = PathBuilder::stroke(stroke_width);
                                    fg2.move_to(bg2_start_pt);
                                    fg2.arc_to(point(radius, radius), px(0.0), arc2_sweep * value2 > PI, true, fg2_end_pt);
                                    if let Ok(path) = fg2.build() {
                                        window.paint_path(path, color);
                                    }
                                }
                            },
                        )
                        .size_full(),
                    )
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
                            .child(label),
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
                            .child(format!("{}%", percent)),
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
