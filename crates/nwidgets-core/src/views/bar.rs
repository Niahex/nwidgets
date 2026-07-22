use gpui::*;
use gpui_component::corner::{Corner, CornerPosition};
use nwidgets_component_active_window::ActiveWindowComponent;
use nwidgets_component_datetime::DateTimeComponent;
use nwidgets_component_pomodoro::PomodoroComponent;
use nwidgets_component_quicksettings::QuickSettingsComponent;

const CORNER_RADIUS: f32 = 12.0;
const BAR_HEIGHT: f32 = 50.0;

pub struct Bar {
    active_window: Entity<ActiveWindowComponent>,
    pomodoro: Entity<PomodoroComponent>,
    quicksettings: Entity<QuickSettingsComponent>,
    datetime: Entity<DateTimeComponent>,
    cc_window: AnyWindowHandle,
    cc_visible: std::rc::Rc<std::cell::Cell<bool>>,
}

impl Bar {
    pub fn new(cc_window: AnyWindowHandle, cx: &mut Context<Self>) -> Self {
        let active_window = cx.new(ActiveWindowComponent::new);
        let pomodoro = cx.new(PomodoroComponent::new);
        let quicksettings = cx.new(QuickSettingsComponent::new);
        let datetime = cx.new(DateTimeComponent::new);
        let cc_visible = std::rc::Rc::new(std::cell::Cell::new(false));

        Self {
            active_window,
            pomodoro,
            quicksettings,
            datetime,
            cc_window,
            cc_visible,
        }
    }
}

impl Render for Bar {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let bg = rgb(0x2e3440);
        let cc_win = self.cc_window;
        let cc_vis = self.cc_visible.clone();

        div()
            .size_full()
            .flex()
            .flex_col()
            .child(
                // ── Bar content ──
                div()
                    .w_full()
                    .h(px(BAR_HEIGHT))
                    .bg(bg)
                    .relative()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_4()
                    // ── Left: Active window ──
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .max_w(px(320.0))
                            .overflow_hidden()
                            .child(self.active_window.clone()),
                    )
                    // ── Center: Pomodoro ──
                    .child(
                        div()
                            .absolute()
                            .inset_0()
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(self.pomodoro.clone()),
                    )
                    // ── Right: QuickSettings & DateTime (Toggle Control Center on click) ──
                    .child(
                        div()
                            .id("quicksettings-trigger")
                            .flex()
                            .gap_3()
                            .items_center()
                            .cursor_pointer()
                            .on_click(move |_event, _window, cx| {
                                let v = !cc_vis.get();
                                cc_vis.set(v);
                                nwidgets_control_center::toggle(&cc_win, v, cx);
                            })
                            .child(self.quicksettings.clone())
                            .child(self.datetime.clone()),
                    ),
            )
            // ── Corners sous la barre ──
            .child(
                div()
                    .w_full()
                    .h(px(CORNER_RADIUS))
                    .flex()
                    .flex_row()
                    .child(
                        // Coin gauche (sous la barre)
                        div().flex_none().child(
                            Corner::new(CornerPosition::TopLeft, px(CORNER_RADIUS)).color(bg),
                        ),
                    )
                    .child(
                        // Espace au milieu (transparent)
                        div().flex_1(),
                    )
                    .child(
                        // Coin droit (sous la barre)
                        div().flex_none().child(
                            Corner::new(CornerPosition::TopRight, px(CORNER_RADIUS)).color(bg),
                        ),
                    ),
            )
    }
}
