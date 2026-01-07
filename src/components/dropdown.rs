use super::PopoverMenu;
use gpui::prelude::*;
use gpui::*;
use std::rc::Rc;

#[derive(IntoElement)]
pub struct Dropdown<T: Clone + PartialEq + 'static> {
    id: ElementId,
    selected: Option<T>,
    options: Vec<DropdownOption<T>>,
    is_open: bool,
    label_fn: Rc<dyn Fn(&T) -> SharedString>,
    placeholder: SharedString,
    on_toggle: Option<Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>>,
    on_select: Option<Rc<dyn Fn(&T, &mut Window, &mut App)>>,
}

#[derive(Clone)]
pub struct DropdownOption<T> {
    pub value: T,
    pub label: SharedString,
}

impl<T: Clone + PartialEq + 'static> Dropdown<T> {
    pub fn new(id: impl Into<ElementId>, options: Vec<DropdownOption<T>>) -> Self {
        Self {
            id: id.into(),
            selected: None,
            options,
            is_open: false,
            label_fn: Rc::new(|_| "".into()),
            placeholder: "Select...".into(),
            on_toggle: None,
            on_select: None,
        }
    }

    pub fn selected(mut self, value: Option<T>) -> Self {
        self.selected = value;
        self
    }

    pub fn label_fn(mut self, f: impl Fn(&T) -> SharedString + 'static) -> Self {
        self.label_fn = Rc::new(f);
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn open(mut self, is_open: bool) -> Self {
        self.is_open = is_open;
        self
    }

    pub fn on_toggle(mut self, f: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_toggle = Some(Rc::new(f));
        self
    }

    pub fn on_select(mut self, f: impl Fn(&T, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(f));
        self
    }
}

impl<T: Clone + PartialEq + 'static> RenderOnce for Dropdown<T> {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<crate::theme::Theme>().clone();
        let is_open = self.is_open;

        let label: SharedString = self
            .selected
            .as_ref()
            .map(|s| (self.label_fn)(s))
            .unwrap_or_else(|| self.placeholder.clone());

        let trigger = div()
            .id(self.id.clone())
            .bg(theme.surface)
            .rounded_md()
            .p_2()
            .flex()
            .items_center()
            .justify_between()
            .cursor_pointer()
            .hover(|s| s.bg(theme.hover))
            .when_some(self.on_toggle.clone(), |this, on_toggle| {
                this.on_click(move |ev, window, cx| on_toggle(ev, window, cx))
            })
            .child(div().text_xs().text_color(theme.text).child(label))
            .child(
                crate::utils::Icon::new(if is_open { "arrow-up" } else { "arrow-down" })
                    .size(px(12.))
                    .color(theme.text_muted),
            );

        let options = self.options.clone();
        let selected = self.selected.clone();
        let on_select = self.on_select.clone();
        let on_toggle = self.on_toggle.clone();

        PopoverMenu::new("dropdown-menu", trigger, move |_window, cx| {
            let theme = cx.global::<crate::theme::Theme>().clone();

            div()
                .flex()
                .flex_col()
                .gap_1()
                .children(options.iter().enumerate().map(|(idx, option)| {
                    let is_selected = selected.as_ref() == Some(&option.value);
                    let value = option.value.clone();
                    let on_select = on_select.clone();
                    let on_toggle = on_toggle.clone();

                    div()
                        .id(("dropdown-option", idx))
                        .flex()
                        .items_center()
                        .gap_2()
                        .px_2()
                        .py_1()
                        .rounded_md()
                        .cursor_pointer()
                        .hover(|s| s.bg(theme.hover))
                        .when(is_selected, |this| this.bg(theme.surface))
                        .when_some(on_select, |this, on_select| {
                            this.on_click(move |ev, window, cx| {
                                on_select(&value, window, cx);
                                if let Some(on_toggle) = &on_toggle {
                                    on_toggle(ev, window, cx);
                                }
                            })
                        })
                        .child(
                            div()
                                .flex_1()
                                .text_xs()
                                .text_color(theme.text)
                                .child(option.label.clone()),
                        )
                        .when(is_selected, |this| {
                            this.child(div().text_xs().text_color(theme.accent).child("✓"))
                        })
                }))
                .into_any_element()
        })
        .open(is_open)
        .on_close(move |window, cx| {
            if let Some(on_toggle) = &self.on_toggle {
                on_toggle(&ClickEvent::default(), window, cx);
            }
        })
    }
}
