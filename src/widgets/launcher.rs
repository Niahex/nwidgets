use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::widgets::*;
    use crate::theme::*;

    SearchResultItem = <View> {
        width: Fill, height: 48
        flow: Right
        align: {x: 0.0, y: 0.5}
        padding: {left: 12, right: 12}
        spacing: 12

        show_bg: true
        draw_bg: {
            instance selected: 0.0

            fn pixel(self) -> vec4 {
                let color = mix(#0000, #4C566A40, self.selected);
                return color;
            }
        }

        icon = <View> {
            width: 32, height: 32
            align: {x: 0.5, y: 0.5}
        }

        info = <View> {
            width: Fill, height: Fit
            flow: Down
            spacing: 2

            name = <Label> {
                draw_text: {
                    color: (THEME_COLOR_TEXT_DEFAULT)
                    text_style: (THEME_FONT_REGULAR) { font_size: 13.0 }
                }
                text: "Application"
            }

            description = <Label> {
                draw_text: {
                    color: (THEME_COLOR_TEXT_MUTE)
                    text_style: (THEME_FONT_REGULAR) { font_size: 11.0 }
                }
                text: ""
            }
        }
    }

    pub Launcher = {{Launcher}} {
        width: 700, height: 500

        show_bg: true
        draw_bg: {
            color: (NORD_POLAR_0)
        }

        flow: Down
        padding: 16

        visible: false

        search_container = <View> {
            width: Fill, height: 48
            flow: Right
            align: {x: 0.0, y: 0.5}
            padding: {left: 16, right: 16}
            spacing: 12

            show_bg: true
            draw_bg: {
                color: (NORD_POLAR_1)
                radius: 8.0
            }

            search_icon = <Label> {
                draw_text: {
                    color: (THEME_COLOR_TEXT_MUTE)
                    text_style: (THEME_FONT_REGULAR) { font_size: 16.0 }
                }
                text: ""
            }

            search_input = <TextInput> {
                width: Fill, height: Fit

                draw_bg: { color: #0000 }
                draw_text: {
                    color: (THEME_COLOR_TEXT_DEFAULT)
                    text_style: (THEME_FONT_REGULAR) { font_size: 14.0 }
                }
                empty_message: "Search apps, calculator (=), processes (ps)..."
            }
        }

        results_container = <View> {
            width: Fill, height: Fill
            flow: Down
            padding: {top: 12}

            results_list = <PortalList> {
                width: Fill, height: Fill

                SearchResultItem = <SearchResultItem> {}
            }
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct Launcher {
    #[deref]
    view: View,

    #[rust]
    query: String,

    #[rust]
    selected_index: usize,

    #[rust]
    results: Vec<LauncherResult>,
}

#[derive(Clone, Debug)]
pub struct LauncherResult {
    pub id: String,
    pub name: String,
    pub description: String,
    pub result_type: LauncherResultType,
}

#[derive(Clone, Debug)]
pub enum LauncherResultType {
    Application,
    Calculator,
    Process,
    Clipboard,
}

impl Widget for Launcher {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        while let Some(item) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = item.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, self.results.len());

                while let Some(item_id) = list.next_visible_item(cx) {
                    if item_id < self.results.len() {
                        let result = &self.results[item_id];
                        let item = list.item(cx, item_id, live_id!(SearchResultItem));

                        item.label(ids!(info.name)).set_text(cx, &result.name);
                        item.label(ids!(info.description)).set_text(cx, &result.description);

                        item.draw_all(cx, &mut Scope::empty());
                    }
                }
            }
        }
        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);

        if let Event::KeyDown(ke) = event {
            match ke.key_code {
                KeyCode::Escape => {
                    cx.widget_action(
                        self.widget_uid(),
                        &HeapLiveIdPath::default(),
                        LauncherAction::Close,
                    );
                }
                KeyCode::ArrowUp => {
                    if self.selected_index > 0 {
                        self.selected_index -= 1;
                        self.view.redraw(cx);
                    }
                }
                KeyCode::ArrowDown => {
                    if self.selected_index < self.results.len().saturating_sub(1) {
                        self.selected_index += 1;
                        self.view.redraw(cx);
                    }
                }
                KeyCode::ReturnKey => {
                    if let Some(result) = self.results.get(self.selected_index) {
                        cx.widget_action(
                            self.widget_uid(),
                            &HeapLiveIdPath::default(),
                            LauncherAction::Launch(result.id.clone()),
                        );
                    }
                }
                _ => {}
            }
        }
    }
}

impl Launcher {
    pub fn set_query(&mut self, _cx: &mut Cx, query: &str) {
        self.query = query.to_string();
        self.selected_index = 0;
    }

    pub fn set_results(&mut self, cx: &mut Cx, results: Vec<LauncherResult>) {
        self.results = results;
        self.selected_index = 0;
        self.view.redraw(cx);
    }

    pub fn show(&mut self, cx: &mut Cx) {
        self.view.apply_over(cx, live! { visible: true });
        self.view.redraw(cx);
    }

    pub fn hide(&mut self, cx: &mut Cx) {
        self.view.apply_over(cx, live! { visible: false });
        self.query.clear();
        self.results.clear();
        self.selected_index = 0;
        self.view.redraw(cx);
    }
}

#[derive(Clone, Debug, DefaultNone)]
pub enum LauncherAction {
    None,
    Close,
    Launch(String),
    QueryChanged(String),
}
