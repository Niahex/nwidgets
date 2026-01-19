use gpui::prelude::*;
use gpui::*;

type ToggleClickHandler = Box<dyn Fn(bool, &mut Window, &mut App) + 'static>;

#[derive(IntoElement)]
pub struct Toggle {
    id: ElementId,
    checked: bool,
    on_click: Option<ToggleClickHandler>,
}

impl Toggle {
    pub fn new(id: impl Into<ElementId>, checked: bool) -> Self {
        Self {
            id: id.into(),
            checked,
            on_click: None,
        }
    }

    pub fn on_click(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for Toggle {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<crate::theme::Theme>();
        let is_on = self.checked;

        let (bg_color, border_color) = if is_on {
            (theme.accent.opacity(0.4), theme.accent.opacity(0.2))
        } else {
            (theme.surface, theme.overlay)
        };

        let bg_hover_color = if is_on {
            theme.accent.opacity(0.5)
        } else {
            theme.hover
        };

        let thumb_color = theme.text;
        let thumb_opacity = if is_on { 1.0 } else { 0.5 };

        let group_id = format!("toggle_{:?}", self.id);

        div()
            .id(self.id)
            .w(px(32.))
            .h(px(20.))
            .rounded_full()
            .p(px(2.))
            .cursor_pointer()
            .group(group_id.clone())
            .child(
                div()
                    .flex()
                    .items_center()
                    .when(is_on, |this| this.justify_end())
                    .when(!is_on, |this| this.justify_start())
                    .size_full()
                    .rounded_full()
                    .px(px(2.))
                    .bg(bg_color)
                    .border_1()
                    .border_color(border_color)
                    .group_hover(group_id.clone(), |el| el.bg(bg_hover_color))
                    .child(
                        div()
                            .size(px(12.))
                            .rounded_full()
                            .bg(thumb_color)
                            .opacity(thumb_opacity),
                    ),
            )
            .when_some(self.on_click, |this, on_click| {
                this.on_click(move |_, window, cx| {
                    on_click(!is_on, window, cx);
                })
            })
    }
}
