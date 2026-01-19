use gpui::prelude::*;
use gpui::*;
use std::f32::consts::PI;

// Import LineCap from lyon_tessellation
use lyon_tessellation::LineCap;

pub struct CircularProgress {
    size: Pixels,
    percent: u8,
    secondary_percent: Option<u8>,
    label: SharedString,
    color: Hsla,
    secondary_color: Hsla,
    primary_unit: SharedString,
    secondary_unit: SharedString,
    display_value: Option<u32>,
    display_secondary_value: Option<u32>,
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
            primary_unit: "".into(),
            secondary_unit: "°C".into(),
            display_value: None,
            display_secondary_value: None,
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

    pub fn primary_unit(mut self, unit: impl Into<SharedString>) -> Self {
        self.primary_unit = unit.into();
        self
    }

    pub fn secondary_unit(mut self, unit: impl Into<SharedString>) -> Self {
        self.secondary_unit = unit.into();
        self
    }

    pub fn display_values(mut self, primary: u32, secondary: u32) -> Self {
        self.display_value = Some(primary);
        self.display_secondary_value = Some(secondary);
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
                    .size(size * 1.25)
                    .relative()
                    .child(
                        canvas(
                            move |bounds, _window, _cx| (bounds, percent, temp, color, temp_color),
                            move |bounds, (_, percent, temp, color, temp_color), window, _cx| {
                                let center = bounds.center();
                                let stroke_width = size * 0.08;
                                let radius = (size - stroke_width) / 2.0;
                                
                                let stroke_opts = StrokeOptions::default()
                                    .with_line_width(stroke_width.into())
                                    .with_line_cap(LineCap::Round);
                                
                                // Gap entre les segments (en degrés)
                                let gap = 10.0;
                                
                                // Arc 1: Température (45° à 220°) - 175° d'espace
                                let arc1_start = 45.0;
                                let arc1_end = 220.0 - gap; // Retirer le gap à la fin
                                let arc1_start_rad = arc1_start * PI / 180.0;
                                let arc1_end_rad = arc1_end * PI / 180.0;
                                let arc1_sweep = arc1_end_rad - arc1_start_rad;
                                
                                // Background arc1
                                let bg1_start_pt = center + point(radius * arc1_start_rad.cos(), radius * arc1_start_rad.sin());
                                let bg1_end_pt = center + point(radius * arc1_end_rad.cos(), radius * arc1_end_rad.sin());
                                let mut bg1 = PathBuilder::default().with_style(PathStyle::Stroke(stroke_opts));
                                bg1.move_to(bg1_start_pt);
                                bg1.arc_to(point(radius, radius), px(0.0), arc1_sweep > PI, true, bg1_end_pt);
                                if let Ok(path) = bg1.build() {
                                    window.paint_path(path, rgb(0x4c566a));
                                }
                                
                                // Foreground arc1 (température)
                                if temp > 0 {
                                    let value1 = (temp as f32 / 100.0).max(1.0 / 360.0);
                                    let fg1_end_angle = arc1_start + (arc1_end - arc1_start) * value1;
                                    let fg1_end_rad = fg1_end_angle * PI / 180.0;
                                    let fg1_end_pt = center + point(radius * fg1_end_rad.cos(), radius * fg1_end_rad.sin());
                                    
                                    let mut fg1 = PathBuilder::default().with_style(PathStyle::Stroke(stroke_opts));
                                    fg1.move_to(bg1_start_pt);
                                    fg1.arc_to(point(radius, radius), px(0.0), (fg1_end_rad - arc1_start_rad) > PI, true, fg1_end_pt);
                                    if let Ok(path) = fg1.build() {
                                        window.paint_path(path, temp_color);
                                    }
                                }
                                
                                // Arc 2: Usage (230° à 360°) - 130° d'espace
                                let arc2_start = 220.0 + gap; // Commencer après le gap
                                let arc2_end = 360.0;
                                let arc2_start_rad = arc2_start * PI / 180.0;
                                let arc2_end_rad = arc2_end * PI / 180.0;
                                let arc2_sweep = arc2_end_rad - arc2_start_rad;
                                
                                // Background arc2
                                let bg2_start_pt = center + point(radius * arc2_start_rad.cos(), radius * arc2_start_rad.sin());
                                let bg2_end_pt = center + point(radius * arc2_end_rad.cos(), radius * arc2_end_rad.sin());
                                let mut bg2 = PathBuilder::default().with_style(PathStyle::Stroke(stroke_opts));
                                bg2.move_to(bg2_start_pt);
                                bg2.arc_to(point(radius, radius), px(0.0), arc2_sweep > PI, true, bg2_end_pt);
                                if let Ok(path) = bg2.build() {
                                    window.paint_path(path, rgb(0x4c566a));
                                }
                                
                                // Foreground arc2 (usage)
                                if percent > 0 {
                                    let value2 = (percent as f32 / 100.0).max(1.0 / 360.0);
                                    let fg2_end_angle = arc2_start + (arc2_end - arc2_start) * value2;
                                    let fg2_end_rad = fg2_end_angle * PI / 180.0;
                                    let fg2_end_pt = center + point(radius * fg2_end_rad.cos(), radius * fg2_end_rad.sin());
                                    
                                    let mut fg2 = PathBuilder::default().with_style(PathStyle::Stroke(stroke_opts));
                                    fg2.move_to(bg2_start_pt);
                                    fg2.arc_to(point(radius, radius), px(0.0), (fg2_end_rad - arc2_start_rad) > PI, true, fg2_end_pt);
                                    if let Ok(path) = fg2.build() {
                                        window.paint_path(path, color);
                                    }
                                }
                            },
                        )
                        .size_full(),
                    )
                    // Label central (charge)
                    .child(
                        div()
                            .absolute()
                            .inset_0()
                            .flex()
                            .items_center()
                            .justify_center()
                            .flex_col()
                            .child(
                                div()
                                    .text_size(size * 0.22)
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(0xeceff4))
                                    .child(if let Some(val) = self.display_value {
                                        format!("{}{}", val, self.primary_unit)
                                    } else {
                                        format!("{}%", percent)
                                    }),
                            )
                            .child(
                                div()
                                    .text_size(size * 0.11)
                                    .text_color(rgb(0xd8dee9))
                                    .child(label),
                            ),
                    )
                    // Température/valeur secondaire en bas à droite (entre les segments)
                    .when(self.secondary_percent.is_some(), |this| {
                        let unit = self.secondary_unit.clone();
                        let value = if let Some(val) = self.display_secondary_value {
                            format!("{}{}", val, unit)
                        } else {
                            format!("{}{}", self.secondary_percent.unwrap(), unit)
                        };
                        this.child(
                            div()
                                .absolute()
                                .right(-size * 0.15)
                                .top(size * 0.7)
                                .text_size(size * 0.15)
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(0xeceff4))
                                .child(value),
                        )
                    }),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_0p5(),
            )
    }
}
