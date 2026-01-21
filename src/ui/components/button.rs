use crate::theme::ActiveTheme;
use gpui::prelude::*;
use gpui::*;
use std::rc::Rc;

#[allow(dead_code)]
type ClickHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;
type MouseDownHandler = Rc<dyn Fn(&MouseDownEvent, &mut Window, &mut App)>;

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum ButtonVariant {
    #[default]
    Default,
    Accent,
    Danger,
    Ghost,
}

#[allow(dead_code)]
#[derive(IntoElement)]
pub struct Button {
    id: ElementId,
    label: Option<SharedString>,
    icon: Option<SharedString>,
    icon_size: Pixels,
    variant: ButtonVariant,
    disabled: bool,
    selected: bool,
    on_click: Option<ClickHandler>,
    on_right_click: Option<MouseDownHandler>,
    on_middle_click: Option<MouseDownHandler>,
    children: Vec<AnyElement>,
}

impl Button {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            label: None,
            icon: None,
            icon_size: px(16.),
            variant: ButtonVariant::Default,
            disabled: false,
            selected: false,
            on_click: None,
            on_right_click: None,
            on_middle_click: None,
            children: Vec::new(),
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn icon(mut self, icon: impl Into<SharedString>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn icon_size(mut self, size: Pixels) -> Self {
        self.icon_size = size;
        self
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn accent(mut self) -> Self {
        self.variant = ButtonVariant::Accent;
        self
    }

    pub fn danger(mut self) -> Self {
        self.variant = ButtonVariant::Danger;
        self
    }

    pub fn ghost(mut self) -> Self {
        self.variant = ButtonVariant::Ghost;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn on_click(mut self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }

    pub fn on_right_click(mut self, handler: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_right_click = Some(Rc::new(handler));
        self
    }

    pub fn on_middle_click(mut self, handler: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_middle_click = Some(Rc::new(handler));
        self
    }

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }
}

impl RenderOnce for Button {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        
        let (bg, text_color, hover_bg, hover_text) = match (self.variant, self.selected) {
            (ButtonVariant::Accent, true) => (
                theme.accent.opacity(0.2),
                theme.accent,
                theme.accent,
                theme.bg,
            ),
            (ButtonVariant::Accent, false) => (
                Hsla::transparent_black(),
                theme.text_muted.opacity(0.5),
                theme.hover,
                theme.text_muted.opacity(0.8),
            ),
            (ButtonVariant::Danger, _) => (
                theme.red.opacity(0.2),
                theme.red,
                theme.red.opacity(0.3),
                theme.red,
            ),
            (ButtonVariant::Ghost, true) => (
                theme.accent.opacity(0.2),
                theme.accent,
                theme.hover,
                theme.accent,
            ),
            (ButtonVariant::Ghost, false) => (
                Hsla::transparent_black(),
                theme.text_muted.opacity(0.5),
                theme.hover,
                theme.text_muted.opacity(0.8),
            ),
            (ButtonVariant::Default, _) => (
                theme.surface,
                theme.text,
                theme.hover,
                theme.text,
            ),
        };

        let font_weight = if self.selected {
            FontWeight::BOLD
        } else {
            FontWeight::MEDIUM
        };

        div()
            .id(self.id)
            .flex()
            .items_center()
            .justify_center()
            .gap_2()
            .px_3()
            .py_2()
            .rounded_md()
            .text_sm()
            .font_weight(font_weight)
            .bg(bg)
            .text_color(text_color)
            .when(!self.disabled, |this| {
                this.cursor_pointer()
                    .hover(move |style| style.bg(hover_bg).text_color(hover_text))
            })
            .when(self.disabled, |this| {
                this.opacity(0.5)
            })
            .when_some(self.on_click, |this, on_click| {
                this.when(!self.disabled, |this| {
                    this.on_click(move |ev, window, cx| on_click(ev, window, cx))
                })
            })
            .when_some(self.on_right_click, |this, on_right_click| {
                this.when(!self.disabled, |this| {
                    this.on_mouse_down(MouseButton::Right, move |ev, window, cx| on_right_click(ev, window, cx))
                })
            })
            .when_some(self.on_middle_click, |this, on_middle_click| {
                this.when(!self.disabled, |this| {
                    this.on_mouse_down(MouseButton::Middle, move |ev, window, cx| on_middle_click(ev, window, cx))
                })
            })
            .when_some(self.icon, |this, icon| {
                this.child(
                    crate::assets::Icon::new(icon)
                        .size(self.icon_size)
                        .color(text_color)
                )
            })
            .when_some(self.label, |this, label| {
                this.child(label)
            })
            .children(self.children)
    }
}
