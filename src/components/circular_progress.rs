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
                                let padding = px(0.0);
                                let spacing = size * 0.05;
                                let arc_radius = (size - padding - stroke_width) / 2.0;
                                
                                // Calcul du gap_angle comme dans le QML
                                let gap_angle = ((spacing + stroke_width) / arc_radius) * (180.0 / PI);
                                
                                // Usage (valeur principale)
                                let start_angle = -90.0;
                                let value = (percent as f32 / 100.0).max(1.0 / 360.0);
                                
                                // Arc de fond (background) - QML ligne 28-42
                                let bg_start = start_angle + 360.0 * value + gap_angle;
                                let bg_sweep = (360.0 * (1.0 - value) - gap_angle * 2.0).max(-gap_angle);
                                
                                if bg_sweep > 0.0 {
                                    let start_rad = bg_start * PI / 180.0;
                                    let end_rad = (bg_start + bg_sweep) * PI / 180.0;
                                    
                                    let start_pt = center + point(arc_radius * start_rad.cos(), arc_radius * start_rad.sin());
                                    let end_pt = center + point(arc_radius * end_rad.cos(), arc_radius * end_rad.sin());
                                    
                                    let mut bg = PathBuilder::stroke(stroke_width);
                                    bg.move_to(start_pt);
                                    bg.arc_to(point(arc_radius, arc_radius), px(0.0), bg_sweep > 180.0, true, end_pt);
                                    if let Ok(path) = bg.build() {
                                        window.paint_path(path, rgb(0x4c566a));
                                    }
                                }
                                
                                // Arc de valeur (foreground) - QML ligne 44-58
                                if percent > 0 {
                                    let fg_sweep = 360.0 * value;
                                    let start_rad = start_angle * PI / 180.0;
                                    let end_rad = (start_angle + fg_sweep) * PI / 180.0;
                                    
                                    let start_pt = center + point(arc_radius * start_rad.cos(), arc_radius * start_rad.sin());
                                    let end_pt = center + point(arc_radius * end_rad.cos(), arc_radius * end_rad.sin());
                                    
                                    let mut fg = PathBuilder::stroke(stroke_width);
                                    fg.move_to(start_pt);
                                    fg.arc_to(point(arc_radius, arc_radius), px(0.0), fg_sweep > 180.0, true, end_pt);
                                    if let Ok(path) = fg.build() {
                                        window.paint_path(path, color);
                                    }
                                }
                                
                                // Température (segment secondaire en haut)
                                if temp > 0 {
                                    let temp_start = 120.0;
                                    let temp_value = (temp as f32 / 100.0).max(1.0 / 360.0);
                                    let temp_sweep = 30.0 * temp_value;
                                    
                                    let start_rad = temp_start * PI / 180.0;
                                    let end_rad = (temp_start - temp_sweep) * PI / 180.0;
                                    
                                    let start_pt = center + point(arc_radius * start_rad.cos(), arc_radius * start_rad.sin());
                                    let end_pt = center + point(arc_radius * end_rad.cos(), arc_radius * end_rad.sin());
                                    
                                    let mut temp_path = PathBuilder::stroke(stroke_width);
                                    temp_path.move_to(start_pt);
                                    temp_path.arc_to(point(arc_radius, arc_radius), px(0.0), false, false, end_pt);
                                    if let Ok(path) = temp_path.build() {
                                        window.paint_path(path, temp_color);
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
