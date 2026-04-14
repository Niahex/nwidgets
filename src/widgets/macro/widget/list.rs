use crate::components::{Button, ButtonVariant};
use crate::theme::Theme;
use crate::widgets::r#macro::service::MacroService;
use gpui::prelude::*;
use gpui::*;

impl super::MacroWidget {
    pub(super) fn render_macro_list(
        &mut self,
        theme: Theme,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let macros = self.macro_service.read(cx).get_macros().clone();
        let playing_id = self.macro_service.read(cx).is_playing();

        div()
            .id("macro-list-scroll")
            .flex()
            .flex_col()
            .flex_1()
            .overflow_y_scroll()
            .gap_2()
            .children(macros.into_iter().map(|macro_rec| {
                let macro_id = macro_rec.id;
                let is_playing = playing_id == Some(macro_id);

                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .p_3()
                    .bg(if is_playing {
                        theme.accent.opacity(0.2)
                    } else {
                        theme.surface
                    })
                    .border_1()
                    .border_color(if is_playing {
                        theme.accent
                    } else {
                        theme.border()
                    })
                    .rounded(px(6.))
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(theme.text)
                                    .child(macro_rec.name.clone()),
                            )
                            .child(
                                div()
                                    .flex()
                                    .gap_2()
                                    .text_xs()
                                    .text_color(theme.text_muted)
                                    .child(format!("{} actions", macro_rec.action_count()))
                                    .child("•")
                                    .child(format!("{}ms", macro_rec.duration_ms()))
                                    .when_some(macro_rec.app_class.clone(), |this, app| {
                                        this.child("•").child(app)
                                    }),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .child(
                                Button::new(format!("play-{}", macro_id))
                                    .icon("play")
                                    .icon_size(px(16.))
                                    .icon_only()
                                    .on_click(cx.listener(move |this, _, _window, cx| {
                                        this.macro_service.update(cx, |service, cx| {
                                            if is_playing {
                                                service.stop_playback(cx);
                                            } else {
                                                service.play_macro(macro_id, cx);
                                            }
                                        });
                                    })),
                            )
                            .child(
                                Button::new(format!("edit-{}", macro_id))
                                    .icon("edit")
                                    .icon_size(px(16.))
                                    .icon_only()
                                    .on_click(cx.listener(move |this, _, _window, cx| {
                                        this.editing_macro_id = Some(macro_id);
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new(format!("delete-{}", macro_id))
                                    .icon("delete")
                                    .icon_size(px(16.))
                                    .icon_only()
                                    .variant(ButtonVariant::Danger)
                                    .on_click(cx.listener(move |this, _, _window, cx| {
                                        this.macro_service.update(cx, |service, cx| {
                                            service.delete_macro(macro_id, cx);
                                        });
                                    })),
                            ),
                    )
            }))
    }
}
