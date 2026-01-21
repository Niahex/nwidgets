use crate::components::CircularProgress;
use crate::theme::ActiveTheme;
use gpui::prelude::*;
use gpui::*;

impl super::super::ControlCenterWidget {
    pub(in crate::widgets::control_center) fn render_monitor_details(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.theme();
        let stats = self.system_monitor.read(cx).stats();

        deferred(
            div()
                .bg(theme.bg)
                .rounded_md()
                .p_3()
                .flex()
                .flex_col()
                .gap_2()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .gap(px(15.))
                        .p_4()
                        .bg(theme.surface)
                        .rounded_md()
                        .child({
                            let mut progress = CircularProgress::new(px(80.)).percent(stats.cpu).label("CPU").color(theme.accent);
                            if let Some(temp) = stats.cpu_temp {
                                progress = progress.secondary_percent(temp as u8).secondary_color(theme.yellow);
                            }
                            progress
                        })
                        .child({
                            let mut progress = CircularProgress::new(px(80.)).percent(stats.gpu).label("GPU").color(theme.accent);
                            if let Some(temp) = stats.gpu_temp {
                                progress = progress.secondary_percent(temp as u8).secondary_color(theme.yellow);
                            }
                            progress
                        })
                        .child({
                            let mut progress = CircularProgress::new(px(80.)).percent(stats.ram).label("Memory").color(theme.accent);
                            if let Some(root_disk) = stats.disks.iter().find(|d| d.mount == "/") {
                                progress = progress.secondary_percent(root_disk.percent).secondary_color(theme.yellow).secondary_unit("%");
                            }
                            progress
                        })
                        .child({
                            let down_mbps = (stats.net_down as f64 * 8.0) / (1024.0 * 1024.0);
                            let up_mbps = (stats.net_up as f64 * 8.0) / (1024.0 * 1024.0);
                            let max_mbps = 1000.0;
                            let down_percent = ((down_mbps / max_mbps * 100.0).min(100.0)) as u8;
                            let up_percent = ((up_mbps / max_mbps * 100.0).min(100.0)) as u8;

                            CircularProgress::new(px(80.))
                                .percent(down_percent)
                                .secondary_percent(up_percent)
                                .label("NET")
                                .color(theme.accent)
                                .secondary_color(theme.purple)
                                .secondary_unit("↑")
                                .display_values(down_mbps as u32, up_mbps as u32)
                                .primary_unit("↓")
                        }),
                )
                .children(stats.metrics().iter().map(|metric| {
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .p_2()
                        .bg(theme.surface)
                        .rounded_md()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(div().flex_1().text_xs().text_color(theme.text).child(metric.name.clone()))
                                .child(
                                    div()
                                        .flex()
                                        .gap_2()
                                        .child(div().text_xs().text_color(theme.text_muted).child(metric.value.clone()))
                                        .when_some(metric.secondary.clone(), |this, secondary| {
                                            this.child(div().text_xs().text_color(theme.text_muted).child(secondary))
                                        }),
                                ),
                        )
                        .when_some(metric.percent, |this, percent| {
                            this.child(div().h(px(20.)).flex().items_center().child(
                                div()
                                    .flex_1()
                                    .h(px(4.))
                                    .bg(theme.hover)
                                    .rounded(px(2.))
                                    .child(div().w(relative(percent as f32 / 100.0)).h_full().bg(theme.accent).rounded(px(2.))),
                            ))
                        })
                }))
                .when(!stats.disks.is_empty(), |this| {
                    this.child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .p_2()
                            .bg(theme.surface)
                            .rounded_md()
                            .children(stats.disks.iter().take(7).map(|disk| {
                                let color = if disk.percent >= 90 {
                                    theme.red
                                } else if disk.percent >= 75 {
                                    theme.yellow
                                } else {
                                    theme.accent
                                };

                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_1()
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_2()
                                            .child(div().flex_1().text_xs().text_color(theme.text).child(disk.mount.clone()))
                                            .child(div().text_xs().text_color(theme.text_muted).child(format!("{}%", disk.percent))),
                                    )
                                    .child(div().h(px(20.)).flex().items_center().child(
                                        div()
                                            .flex_1()
                                            .h(px(4.))
                                            .bg(theme.hover)
                                            .rounded(px(2.))
                                            .child(div().w(relative(disk.percent as f32 / 100.0)).h_full().bg(color).rounded(px(2.))),
                                    ))
                            })),
                    )
                })
        )
        .into_any_element()
    }
}
