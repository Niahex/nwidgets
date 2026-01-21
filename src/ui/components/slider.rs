use crate::theme::ActiveTheme;
use crate::ui::components::element_ext::ElementExt;
use gpui::prelude::*;
use gpui::*;

#[derive(Clone)]
struct DragThumb(EntityId);

impl Render for DragThumb {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        Empty
    }
}

/// Events emitted by the SliderState
pub enum SliderEvent {
    Change(f32),
}

impl EventEmitter<SliderEvent> for SliderState {}

/// State of the Slider
pub struct SliderState {
    min: f32,
    max: f32,
    value: f32,
    percentage: f32,
    bounds: Bounds<Pixels>,
}

impl SliderState {
    pub fn new(min: f32, max: f32, value: f32) -> Self {
        let mut state = Self {
            min,
            max,
            value: value.clamp(min, max),
            percentage: 0.0,
            bounds: Bounds::default(),
        };
        state.update_percentage();
        state
    }

    pub fn set_value(&mut self, value: f32, _: &mut Window, cx: &mut Context<Self>) {
        self.value = value.clamp(self.min, self.max);
        self.update_percentage();
        cx.emit(SliderEvent::Change(self.value));
        cx.notify();
    }

    pub fn value(&self) -> f32 {
        self.value
    }

    fn update_percentage(&mut self) {
        let range = self.max - self.min;
        self.percentage = if range > 0.0 {
            ((self.value - self.min) / range).clamp(0.0, 1.0)
        } else {
            0.0
        };
    }

    fn update_value_by_position(
        &mut self,
        position: Point<Pixels>,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let inner_pos = position.x - self.bounds.left();
        let total_width = self.bounds.size.width;
        let percentage = (inner_pos / total_width).clamp(0.0, 1.0);
        
        self.percentage = percentage;
        self.value = self.min + (self.max - self.min) * percentage;
        
        cx.emit(SliderEvent::Change(self.value));
        cx.notify();
    }
}

/// A Slider element
#[derive(IntoElement)]
pub struct Slider {
    state: Entity<SliderState>,
    disabled: bool,
}

impl Slider {
    pub fn new(state: &Entity<SliderState>) -> Self {
        Self {
            state: state.clone(),
            disabled: false,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl RenderOnce for Slider {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let entity_id = self.state.entity_id();
        let state = self.state.read(cx);
        let percentage = state.percentage;

        div()
            .relative()
            .w_full()
            .h(px(6.))
            .flex()
            .items_center()
            .child(
                div()
                    .id("slider-container")
                    .w_full()
                    .h(px(6.))
                    .flex()
                    .items_center()
                    .when(!self.disabled, |this| {
                        this.on_mouse_down(
                            MouseButton::Left,
                            window.listener_for(&self.state, move |state, e: &MouseDownEvent, window, cx| {
                                state.update_value_by_position(e.position, window, cx);
                            }),
                        )
                    })
                    .child(
                        div()
                            .id("slider-bar")
                            .relative()
                            .w_full()
                            .h(px(4.))
                            .rounded_full()
                            .bg(theme.surface)
                            .child(
                                // Filled track
                                div()
                                    .absolute()
                                    .h_full()
                                    .rounded_full()
                                    .bg(theme.accent)
                                    .w(relative(percentage))
                            )
                            .child(
                                // Thumb
                                div()
                                    .id("slider-thumb")
                                    .absolute()
                                    .size(px(12.))
                                    .rounded_full()
                                    .bg(theme.accent)
                                    .left(relative(percentage))
                                    .ml(px(-6.))
                                    .top(px(-4.))
                                    .when(!self.disabled, |this| {
                                        this.cursor_pointer()
                                            .on_mouse_down(MouseButton::Left, |_, _, cx| {
                                                cx.stop_propagation();
                                            })
                                            .on_drag(DragThumb(entity_id), |drag, _, _, cx| {
                                                cx.stop_propagation();
                                                cx.new(|_| drag.clone())
                                            })
                                            .on_drag_move(window.listener_for(
                                                &self.state,
                                                move |state, e: &DragMoveEvent<DragThumb>, window, cx| {
                                                    match e.drag(cx) {
                                                        DragThumb(id) => {
                                                            if *id == entity_id {
                                                                state.update_value_by_position(e.event.position, window, cx);
                                                            }
                                                        }
                                                    }
                                                },
                                            ))
                                    })
                            )
                            .on_prepaint({
                                let state = self.state.clone();
                                move |bounds, _, cx| state.update(cx, |r, _| r.bounds = bounds)
                            }),
                    )
            )
    }
}
