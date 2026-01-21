use crate::theme::ActiveTheme;
use gpui::prelude::*;
use gpui::*;
use std::rc::Rc;

type ContentFn = Rc<dyn Fn(&mut Window, &mut App) -> AnyElement>;
type CloseFn = Rc<dyn Fn(&mut Window, &mut App)>;

#[derive(IntoElement)]
pub struct PopoverMenu {
    id: ElementId,
    trigger: AnyElement,
    content: ContentFn,
    is_open: bool,
    on_close: Option<CloseFn>,
}

impl PopoverMenu {
    pub fn new(
        id: impl Into<ElementId>,
        trigger: impl IntoElement,
        content: impl Fn(&mut Window, &mut App) -> AnyElement + 'static,
    ) -> Self {
        Self {
            id: id.into(),
            trigger: trigger.into_any_element(),
            content: Rc::new(content),
            is_open: false,
            on_close: None,
        }
    }

    pub fn open(mut self, is_open: bool) -> Self {
        self.is_open = is_open;
        self
    }

    pub fn on_close(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_close = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for PopoverMenu {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let on_close = self.on_close.clone();
        let is_open = self.is_open;
        let content = self.content.clone();
        let id = self.id.clone();

        div().w_full().child(
            div()
                .id(self.id.clone())
                .relative()
                .w_full()
                .child(self.trigger)
                .when(is_open, move |this| {
                    this.child(
                        deferred(
                            div()
                                .occlude()
                                .absolute()
                                .top_full()
                                .left_0()
                                .w_full()
                                .child(
                                    div()
                                        .id(id)
                                        .mt_1()
                                        .w_full()
                                        .bg(theme.bg)
                                        .border_1()
                                        .border_color(theme.hover)
                                        .rounded_md()
                                        .shadow_lg()
                                        .p_1()
                                        .on_mouse_down_out(move |_, window, cx| {
                                            if let Some(on_close) = &on_close {
                                                on_close(window, cx);
                                            }
                                        })
                                        .child(content(window, cx)),
                                ),
                        )
                        .with_priority(1),
                    )
                }),
        )
    }
}
