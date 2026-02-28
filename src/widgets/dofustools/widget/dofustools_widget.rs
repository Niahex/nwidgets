use crate::services::cef::BrowserView;
use crate::theme::ActiveTheme;
use crate::widgets::dofustools::crafter::CrafterWidget;
use crate::widgets::dofustools::service::DofusToolsService;
use crate::widgets::dofustools::types::DofusToolsToggled;
use gpui::prelude::*;
use gpui::{
    div, Animation, AnimationExt, AppContext, Context, Entity, IntoElement, ParentElement, Styled,
    Window,
};

#[derive(Clone, Copy, PartialEq)]
enum DofusToolsView {
    DofusDb,
    FutureApp,
}

pub struct DofusToolsWidget {
    browser: Entity<BrowserView>,
    dofustools_service: Entity<DofusToolsService>,
    crafter_widget: Entity<CrafterWidget>,
    current_view: DofusToolsView,
}

impl DofusToolsWidget {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let url = "https://dofusdb.fr";
        let browser = cx.new(|cx| BrowserView::new(url, 600, 1370, None, cx));
        let dofustools_service = DofusToolsService::global(cx);
        let crafter_widget = cx.new(CrafterWidget::new);
        browser.read(cx).set_hidden(true);
        let browser_clone = browser.clone();
        cx.subscribe(
            &dofustools_service,
            move |_this, service, _event: &DofusToolsToggled, cx| {
                let visible = service.read(cx).visible;
                browser_clone.read(cx).set_hidden(!visible);
                cx.notify();
            },
        )
        .detach();
        Self {
            browser,
            dofustools_service,
            crafter_widget,
            current_view: DofusToolsView::DofusDb,
        }
    }

    pub fn resize_browser(&self, width: u32, height: u32, cx: &gpui::App) {
        self.browser.read(cx).resize(width, height);
    }

    fn switch_to_view(&mut self, view: DofusToolsView, cx: &mut Context<Self>) {
        self.current_view = view;
        cx.notify();
    }
}

impl gpui::Render for DofusToolsWidget {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let visible = self.dofustools_service.read(cx).visible;

        if !visible {
            return div().into_any_element();
        }

        let theme = cx.theme();
        let nav_height = gpui::px(40.);

        div()
            .id("dofustools-root")
            .size_full()
            .occlude()
            .bg(theme.bg)
            .rounded(gpui::px(18.))
            .overflow_hidden()
            .border_1()
            .border_color(theme.accent_alt.opacity(0.25))
            .shadow_lg()
            .flex()
            .flex_col()
            .child(
                div()
                    .id("nav-bar")
                    .w_full()
                    .h(nav_height)
                    .flex()
                    .flex_row()
                    .bg(theme.surface)
                    .border_b_1()
                    .border_color(theme.accent_alt.opacity(0.25))
                    .child(
                        div()
                            .id("btn-dofusdb")
                            .flex_1()
                            .h_full()
                            .flex()
                            .items_center()
                            .justify_center()
                            .when(self.current_view == DofusToolsView::DofusDb, |this| {
                                this.bg(theme.accent.opacity(0.2))
                                    .border_b_2()
                                    .border_color(theme.accent)
                            })
                            .hover(|this| this.bg(theme.accent.opacity(0.1)))
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.switch_to_view(DofusToolsView::DofusDb, cx);
                            }))
                            .child(
                                div()
                                    .text_color(if self.current_view == DofusToolsView::DofusDb {
                                        theme.accent
                                    } else {
                                        theme.text_muted
                                    })
                                    .child("DofusDB"),
                            ),
                    )
                    .child(
                        div()
                            .id("btn-tools")
                            .flex_1()
                            .h_full()
                            .flex()
                            .items_center()
                            .justify_center()
                            .when(self.current_view == DofusToolsView::FutureApp, |this| {
                                this.bg(theme.accent.opacity(0.2))
                                    .border_b_2()
                                    .border_color(theme.accent)
                            })
                            .hover(|this| this.bg(theme.accent.opacity(0.1)))
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.switch_to_view(DofusToolsView::FutureApp, cx);
                            }))
                            .child(
                                div()
                                    .text_color(if self.current_view == DofusToolsView::FutureApp {
                                        theme.accent
                                    } else {
                                        theme.text_muted
                                    })
                                    .child("Tools"),
                            ),
                    ),
            )
            .child(
                div()
                    .id("content-area")
                    .flex_1()
                    .w_full()
                    .when(self.current_view == DofusToolsView::DofusDb, |this| {
                        this.child(self.browser.clone())
                    })
                    .when(self.current_view == DofusToolsView::FutureApp, |this| {
                        this.child(self.crafter_widget.clone())
                    }),
            )
            .with_animation(
                "dofustools-fade-in",
                Animation::new(std::time::Duration::from_millis(150)),
                |this, delta| this.opacity(delta),
            )
            .into_any_element()
    }
}
