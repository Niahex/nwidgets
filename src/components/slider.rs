use std::ops::Range;

use gpui::{
    prelude::FluentBuilder as _, px, div, App, AppContext, Axis, Background, Bounds, Context, Corners,
    DragMoveEvent, Empty, Entity, EntityId, EventEmitter, Hsla, InteractiveElement, IntoElement,
    MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, ParentElement as _, Pixels, Point, Render, RenderOnce,
    StatefulInteractiveElement as _, StyleRefinement, Styled, Window,
};

use super::element_ext::ElementExt;

fn h_flex() -> gpui::Div {
    div().flex().flex_row()
}

/// Events emitted by the [`SliderState`].
pub enum SliderEvent {
    Change(SliderValue),
}

/// The value of the slider, can be a single value or a range of values.
///
/// - Can from a f32 value, which will be treated as a single value.
/// - Or from a (f32, f32) tuple, which will be treated as a range of values.
///
/// The default value is `SliderValue::Single(0.0)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SliderValue {
    Single(f32),
    Range(f32, f32),
}

impl std::fmt::Display for SliderValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SliderValue::Single(value) => write!(f, "{}", value),
            SliderValue::Range(start, end) => write!(f, "{}..{}", start, end),
        }
    }
}

impl From<f32> for SliderValue {
    fn from(value: f32) -> Self {
        SliderValue::Single(value)
    }
}

impl From<(f32, f32)> for SliderValue {
    fn from(value: (f32, f32)) -> Self {
        SliderValue::Range(value.0, value.1)
    }
}

impl From<Range<f32>> for SliderValue {
    fn from(value: Range<f32>) -> Self {
        SliderValue::Range(value.start, value.end)
    }
}

impl Default for SliderValue {
    fn default() -> Self {
        SliderValue::Single(0.)
    }
}

impl SliderValue {
    /// Clamp the value to the given range.
    pub fn clamp(self, min: f32, max: f32) -> Self {
        match self {
            SliderValue::Single(value) => SliderValue::Single(value.clamp(min, max)),
            SliderValue::Range(start, end) => {
                SliderValue::Range(start.clamp(min, max), end.clamp(min, max))
            }
        }
    }

    /// Check if the value is a single value.
    #[inline]
    pub fn is_single(&self) -> bool {
        matches!(self, SliderValue::Single(_))
    }

    /// Check if the value is a range of values.
    #[inline]
    pub fn is_range(&self) -> bool {
        matches!(self, SliderValue::Range(_, _))
    }

    /// Get the start value.
    pub fn start(&self) -> f32 {
        match self {
            SliderValue::Single(value) => *value,
            SliderValue::Range(start, _) => *start,
        }
    }

    /// Get the end value.
    pub fn end(&self) -> f32 {
        match self {
            SliderValue::Single(value) => *value,
            SliderValue::Range(_, end) => *end,
        }
    }

    fn set_start(&mut self, value: f32) {
        if let SliderValue::Range(_, end) = self {
            *self = SliderValue::Range(value.min(*end), *end);
        } else {
            *self = SliderValue::Single(value);
        }
    }

    fn set_end(&mut self, value: f32) {
        if let SliderValue::Range(start, _) = self {
            *self = SliderValue::Range(*start, value.max(*start));
        } else {
            *self = SliderValue::Single(value);
        }
    }
}

/// The scale mode of the slider.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SliderScale {
    /// Linear scale where values change uniformly across the slider range.
    /// This is the default mode.
    #[default]
    Linear,
    /// Logarithmic scale where the distance between values increases exponentially.
    ///
    /// This is useful for parameters that have a large range of values where smaller
    /// changes are more significant at lower values. Common examples include:
    ///
    /// - Volume controls (human hearing perception is logarithmic)
    /// - Frequency controls (musical notes follow a logarithmic scale)
    /// - Zoom levels
    /// - Any parameter where you want finer control at lower values
    ///
    /// # For example
    ///
    /// ```
    /// use gpui_component::slider::{SliderState, SliderScale};
    ///
    /// let slider = SliderState::new()
    ///     .min(1.0)    // Must be > 0 for logarithmic scale
    ///     .max(1000.0)
    ///     .scale(SliderScale::Logarithmic);
    /// ```
    ///
    /// - Moving the slider 1/3 of the way will yield ~10
    /// - Moving it 2/3 of the way will yield ~100
    /// - The full range covers 3 orders of magnitude evenly
    Logarithmic,
}

impl SliderScale {
    #[inline]
    pub fn is_linear(&self) -> bool {
        matches!(self, SliderScale::Linear)
    }

    #[inline]
    pub fn is_logarithmic(&self) -> bool {
        matches!(self, SliderScale::Logarithmic)
    }
}

/// State of the [`Slider`].
pub struct SliderState {
    min: f32,
    max: f32,
    step: f32,
    value: SliderValue,
    /// When is single value mode, only `end` is used, the start is always 0.0.
    percentage: Range<f32>,
    /// The bounds of the slider after rendered.
    bounds: Bounds<Pixels>,
    scale: SliderScale,
    /// Track if currently dragging
    dragging: bool,
    /// Track which thumb is being dragged (for range sliders)
    dragging_start: bool,
}

impl SliderState {
    /// Create a new [`SliderState`].
    pub fn new() -> Self {
        Self {
            min: 0.0,
            max: 100.0,
            step: 1.0,
            value: SliderValue::default(),
            percentage: (0.0..0.0),
            bounds: Bounds::default(),
            scale: SliderScale::default(),
            dragging: false,
            dragging_start: false,
        }
    }

    /// Set the minimum value of the slider, default: 0.0
    pub fn min(mut self, min: f32) -> Self {
        if self.scale.is_logarithmic() {
            assert!(
                min > 0.0,
                "`min` must be greater than 0 for SliderScale::Logarithmic"
            );
            assert!(
                min < self.max,
                "`min` must be less than `max` for Logarithmic scale"
            );
        }
        self.min = min;
        self.update_thumb_pos();
        self
    }

    /// Set the maximum value of the slider, default: 100.0
    pub fn max(mut self, max: f32) -> Self {
        if self.scale.is_logarithmic() {
            assert!(
                max > self.min,
                "`max` must be greater than `min` for Logarithmic scale"
            );
        }
        self.max = max;
        self.update_thumb_pos();
        self
    }

    /// Set the step value of the slider, default: 1.0
    pub fn step(mut self, step: f32) -> Self {
        self.step = step;
        self
    }

    /// Set the scale of the slider, default: [`SliderScale::Linear`].
    pub fn scale(mut self, scale: SliderScale) -> Self {
        if scale.is_logarithmic() {
            assert!(
                self.min > 0.0,
                "`min` must be greater than 0 for Logarithmic scale"
            );
            assert!(
                self.max > self.min,
                "`max` must be greater than `min` for Logarithmic scale"
            );
        }
        self.scale = scale;
        self.update_thumb_pos();
        self
    }

    /// Set the default value of the slider, default: 0.0
    pub fn default_value(mut self, value: impl Into<SliderValue>) -> Self {
        self.value = value.into();
        self.update_thumb_pos();
        self
    }

    /// Set the value of the slider.
    pub fn set_value(
        &mut self,
        value: impl Into<SliderValue>,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.value = value.into();
        self.update_thumb_pos();
        cx.notify();
    }

    /// Get the value of the slider.
    pub fn value(&self) -> SliderValue {
        self.value
    }

    /// Converts a value between 0.0 and 1.0 to a value between the minimum and maximum value,
    /// depending on the chosen scale.
    fn percentage_to_value(&self, percentage: f32) -> f32 {
        match self.scale {
            SliderScale::Linear => self.min + (self.max - self.min) * percentage,
            SliderScale::Logarithmic => {
                // when percentage is 0, this simplifies to (max/min)^0 * min = 1 * min = min
                // when percentage is 1, this simplifies to (max/min)^1 * min = (max*min)/min = max
                // we clamp just to make sure we don't have issue with floating point precision
                let base = self.max / self.min;
                (base.powf(percentage) * self.min).clamp(self.min, self.max)
            }
        }
    }

    /// Converts a value between the minimum and maximum value to a value between 0.0 and 1.0,
    /// depending on the chosen scale.
    fn value_to_percentage(&self, value: f32) -> f32 {
        match self.scale {
            SliderScale::Linear => {
                let range = self.max - self.min;
                if range <= 0.0 {
                    0.0
                } else {
                    (value - self.min) / range
                }
            }
            SliderScale::Logarithmic => {
                let base = self.max / self.min;
                (value / self.min).log(base).clamp(0.0, 1.0)
            }
        }
    }

    fn update_thumb_pos(&mut self) {
        match self.value {
            SliderValue::Single(value) => {
                let percentage = self.value_to_percentage(value.clamp(self.min, self.max));
                self.percentage = 0.0..percentage;
            }
            SliderValue::Range(start, end) => {
                let clamped_start = start.clamp(self.min, self.max);
                let clamped_end = end.clamp(self.min, self.max);
                self.percentage =
                    self.value_to_percentage(clamped_start)..self.value_to_percentage(clamped_end);
            }
        }
    }

    /// Update value by mouse position
    fn update_value_by_position(
        &mut self,
        axis: Axis,
        position: Point<Pixels>,
        is_start: bool,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let bounds = self.bounds;
        let step = self.step;

        let inner_pos = match axis {
            Axis::Horizontal => position.x - bounds.left(),
            Axis::Vertical => bounds.bottom() - position.y,
        };
        let total_size = match axis {
            Axis::Horizontal => bounds.size.width,
            Axis::Vertical => bounds.size.height,
        };
        let mut percentage = inner_pos.clamp(px(0.), total_size) / total_size;

        percentage = if is_start {
            percentage.clamp(0.0, self.percentage.end)
        } else {
            percentage.clamp(self.percentage.start, 1.0)
        };

        let value = self.percentage_to_value(percentage);
        let value = (value / step).round() * step;

        let old_value = self.value;
        
        // Update percentage for smooth visual movement
        if is_start {
            self.percentage.start = percentage;
        } else {
            self.percentage.end = percentage;
        }
        
        // Update value (snapped to step)
        if is_start {
            self.value.set_start(value);
        } else {
            self.value.set_end(value);
        }
        
        // Emit event only if value changed
        if old_value != self.value {
            cx.emit(SliderEvent::Change(self.value));
        }
        
        // Always notify for visual update
        cx.notify();
    }
}

impl EventEmitter<SliderEvent> for SliderState {}
impl Render for SliderState {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        Empty
    }
}

/// A Slider element.
#[derive(IntoElement)]
pub struct Slider {
    state: Entity<SliderState>,
    axis: Axis,
    style: StyleRefinement,
    disabled: bool,
}

impl Slider {
    /// Create a new [`Slider`] element bind to the [`SliderState`].
    pub fn new(state: &Entity<SliderState>) -> Self {
        Self {
            axis: Axis::Horizontal,
            state: state.clone(),
            style: StyleRefinement::default(),
            disabled: false,
        }
    }

    /// As a horizontal slider.
    pub fn horizontal(mut self) -> Self {
        self.axis = Axis::Horizontal;
        self
    }

    /// As a vertical slider.
    pub fn vertical(mut self) -> Self {
        self.axis = Axis::Vertical;
        self
    }

    /// Set the disabled state of the slider, default: false
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    fn render_thumb(
        &self,
        start_pos: Pixels,
        is_start: bool,
        bar_color: Background,
        thumb_color: Hsla,
        _window: &mut Window,
        _cx: &mut App,
    ) -> impl gpui::IntoElement {
        let axis = self.axis;
        let id = ("slider-thumb", is_start as u32);

        if self.disabled {
            return div().id(id);
        }

        div()
            .id(id)
            .absolute()
            .when(matches!(axis, Axis::Horizontal), |this| {
                this.top(px(-5.)).left(start_pos).ml(-px(8.))
            })
            .when(matches!(axis, Axis::Vertical), |this| {
                this.bottom(start_pos).left(px(-5.)).mb(-px(8.))
            })
            .flex()
            .items_center()
            .justify_center()
            .flex_shrink_0()
            .rounded_full()
            .bg(bar_color.opacity(0.5))
            .shadow_md()
            .size_4()
            .p(px(1.))
            .child(
                div()
                    .flex_shrink_0()
                    .size_full()
                    .rounded_full()
                    .bg(thumb_color),
            )
    }
}

impl Styled for Slider {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for Slider {
    fn render(self, window: &mut Window, cx: &mut gpui::App) -> impl IntoElement {
        let axis = self.axis;
        let entity_id = self.state.entity_id();
        let state = self.state.read(cx);
        let is_range = state.value().is_range();
        let bar_size = match axis {
            Axis::Horizontal => state.bounds.size.width,
            Axis::Vertical => state.bounds.size.height,
        };
        let bar_start = state.percentage.start * bar_size;
        let bar_end = state.percentage.end * bar_size;
        let rem_size = window.rem_size();

        let bar_color = self
            .style
            .background
            .clone()
            .and_then(|bg| bg.color())
            .unwrap_or_else(|| cx.global::<crate::theme::Theme>().hover.into());
        let thumb_color = self
            .style
            .text
            .as_ref()
            .and_then(|t| t.color)
            .unwrap_or_else(|| cx.global::<crate::theme::Theme>().accent_alt);
        let corner_radii = self.style.corner_radii.clone();
        let default_radius = px(999.);
        let _radius = Corners {
            top_left: corner_radii
                .top_left
                .map(|v| v.to_pixels(rem_size))
                .unwrap_or(default_radius),
            top_right: corner_radii
                .top_right
                .map(|v| v.to_pixels(rem_size))
                .unwrap_or(default_radius),
            bottom_left: corner_radii
                .bottom_left
                .map(|v| v.to_pixels(rem_size))
                .unwrap_or(default_radius),
            bottom_right: corner_radii
                .bottom_right
                .map(|v| v.to_pixels(rem_size))
                .unwrap_or(default_radius),
        };

        // Setup mouse event handlers like scrollbar
        window.on_mouse_event({
            let state = self.state.clone();
            move |event: &MouseMoveEvent, _phase, window, cx| {
                state.update(cx, |state, cx| {
                    if state.dragging {
                        state.update_value_by_position(
                            axis,
                            event.position,
                            state.dragging_start,
                            window,
                            cx,
                        );
                        cx.stop_propagation();
                    }
                });
            }
        });

        window.on_mouse_event({
            let state = self.state.clone();
            move |_event: &MouseUpEvent, _phase, _window, cx| {
                state.update(cx, |state, _cx| {
                    state.dragging = false;
                });
            }
        });

        div()
            .id(("slider", self.state.entity_id()))
            .flex()
            .flex_1()
            .items_center()
            .justify_center()
            .when(matches!(axis, Axis::Vertical), |this| this.h(px(120.)))
            .when(matches!(axis, Axis::Horizontal), |this| this.w_full())
            .child(
                h_flex()
                    .id("slider-bar-container")
                    .when(!self.disabled, |this| {
                        this.on_mouse_down(
                            MouseButton::Left,
                            window.listener_for(
                                &self.state,
                                move |state, e: &MouseDownEvent, window, cx| {
                                    let mut is_start = false;
                                    if is_range {
                                        let inner_pos = match axis {
                                            Axis::Horizontal => e.position.x - state.bounds.left(),
                                            Axis::Vertical => state.bounds.bottom() - e.position.y,
                                        };
                                        let center = (bar_end - bar_start) / 2.0 + bar_start;
                                        is_start = inner_pos < center;
                                    }

                                    state.dragging = true;
                                    state.dragging_start = is_start;
                                    state.update_value_by_position(
                                        axis, e.position, is_start, window, cx,
                                    );
                                },
                            ),
                        )
                    })
                    .when(matches!(axis, Axis::Horizontal), |this| {
                        this.items_center().h_6().w_full()
                    })
                    .when(matches!(axis, Axis::Vertical), |this| {
                        this.justify_center().w_6().h_full()
                    })
                    .flex_shrink_0()
                    .child(
                        div()
                            .id("slider-bar")
                            .relative()
                            .when(matches!(axis, Axis::Horizontal), |this| this.w_full().h(px(6.)))
                            .when(matches!(axis, Axis::Vertical), |this| this.h_full().w(px(6.)))
                            .bg(bar_color.opacity(0.2))
                            .active(|this| this.bg(bar_color.opacity(0.4)))
                            .rounded_full()
                            .child(
                                div()
                                    .absolute()
                                    .when(matches!(axis, Axis::Horizontal), |this| {
                                        this.h_full().left(bar_start).w(bar_end - bar_start)
                                    })
                                    .when(matches!(axis, Axis::Vertical), |this| {
                                        this.w_full().bottom(bar_start).h(bar_end - bar_start)
                                    })
                                    .bg(bar_color)
                                    .rounded_full(),
                            )
                            .when(is_range, |this| {
                                this.child(self.render_thumb(
                                    bar_start,
                                    true,
                                    bar_color,
                                    thumb_color,
                                    window,
                                    cx,
                                ))
                            })
                            .child(self.render_thumb(
                                bar_end,
                                false,
                                bar_color,
                                thumb_color,
                                window,
                                cx,
                            ))
                            .on_prepaint({
                                let state = self.state.clone();
                                move |bounds, _, cx| state.update(cx, |r, _| r.bounds = bounds)
                            }),
                    ),
            )
    }
}
