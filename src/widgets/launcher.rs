use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::widgets::*;
    use makepad_draw::shader::std::*;
    use crate::theme::*;
    use crate::ui::components::list::List;

    SearchResultItem = <View> {
        width: Fill, height: Fit
        flow: Right
        align: {x: 0.0, y: 0.5}
        padding: {top: 8, bottom: 8, left: 12, right: 12}
        spacing: 12

        show_bg: true
        draw_bg: {
            instance selected: 0.0

            fn pixel(self) -> vec4 {
                let color = mix(#0000, #4C566A40, self.selected);
                return color;
            }
        }

        icon = <Icon> {
            width: 24, height: 24
            icon_walk: { width: 24, height: 24 }
            draw_icon: {
                svg_file: dep("crate://self/assets/icons/none.svg")
                brightness: 1.0
                curve: 0.6
                color: #fff
                preserve_colors: true
            }
        }

        info = <View> {
            width: Fill, height: Fit
            flow: Down
            spacing: 2

            name = <Label> {
                draw_text: { text_style: <THEME_FONT_REGULAR> { font_size: 13.0 }, color: (THEME_COLOR_TEXT_DEFAULT) }
                text: "Application"
            }

            description = <Label> {
                draw_text: { text_style: <THEME_FONT_REGULAR> { font_size: 11.0 }, color: (THEME_COLOR_TEXT_MUTE) }
                text: ""
            }
        }
    }

    pub Launcher = {{Launcher}} {
        width: 700, height: 500

        show_bg: true
        draw_bg: {
            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, 16.0);
                sdf.fill((NORD_POLAR_0));
                return sdf.result;
            }
        }

        flow: Down
        padding: 16
        spacing: 16

        visible: false

        search_container = <View> {
            width: Fill, height: 48
            flow: Right
            align: {x: 0.0, y: 0.5}
            padding: {left: 16, right: 16}
            spacing: 12

            show_bg: true
            draw_bg: {
                fn pixel(self) -> vec4 {
                    let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                    sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, 8.0);
                    sdf.fill((NORD_POLAR_1));
                    return sdf.result;
                }
            }

            search_icon = <Icon> {
                width: 24, height: 24
                icon_walk: { width: 24, height: 24 }
                draw_icon: {
                    svg_file: dep("crate://self/assets/icons/search.svg")
                    brightness: 1.0
                    curve: 0.6
                    color: #fff
                }
            }

            search_input = <TextInput> {
                width: Fill, height: Fit

                draw_bg: { color: #0000 }
                draw_text: { text_style: <THEME_FONT_REGULAR> { font_size: 14.0 }, color: (THEME_COLOR_TEXT_DEFAULT) }
                empty_text: "Search apps, calculator (=), processes (ps)..."
            }
        }

        list = <List> {
            height: Fill, width: Fill
            flow: Down

            clip_x: true, clip_y: true

            item0 = <SearchResultItem> {visible: false}
            item1 = <SearchResultItem> {visible: false}
            item2 = <SearchResultItem> {visible: false}
            item3 = <SearchResultItem> {visible: false}
            item4 = <SearchResultItem> {visible: false}
            item5 = <SearchResultItem> {visible: false}
            item6 = <SearchResultItem> {visible: false}
            item7 = <SearchResultItem> {visible: false}
            item8 = <SearchResultItem> {visible: false}
            item9 = <SearchResultItem> {visible: false}
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

    #[rust]
    frames_until_focus: u8,
}


#[derive(Clone, Debug)]
pub struct LauncherResult {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon_path: Option<String>,
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
        self.view.draw_walk(cx, scope, walk)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        // Handle frame countdown for focus
        if let Event::NextFrame(_) = event {
            if self.frames_until_focus > 0 && self.view.visible {
                self.frames_until_focus -= 1;
                if self.frames_until_focus == 0 {
                    ::log::info!("Setting key focus on text input after 2 frames delay");
                    let text_input = self.view.text_input(ids!(search_input));
                    text_input.set_key_focus(cx);
                } else {
                    cx.new_next_frame();
                }
            }
        }

        // Let view handle events first and capture actions
        for action in cx.capture_actions(|cx| self.view.handle_event(cx, event, scope)) {
            if let TextInputAction::Changed(new_text) = action.as_widget_action().cast() {
                self.search(cx, &new_text);
                self.update_results(cx);
            }
        }

        // Handle special keyboard events for navigation
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
                        self.update_results(cx);
                        self.view.redraw(cx);
                    }
                }
                KeyCode::ArrowDown => {
                    if self.selected_index < self.results.len().saturating_sub(1) {
                        self.selected_index += 1;
                        self.update_results(cx);
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
    fn search(&mut self, cx: &mut Cx, query: &str) {
        use crate::APPLICATIONS_SERVICE;

        self.query = query.to_string();
        self.selected_index = 0;

        let mut results = Vec::new();

        if query.starts_with('=') {
            let expr = &query[1..];
            if !expr.is_empty() {
                results.push(LauncherResult {
                    id: format!("calc:{}", expr),
                    name: format!("= {}", expr),
                    description: "Calculator".to_string(),
                    icon_path: Some("./assets/icons/calculator.svg".to_string()),
                    result_type: LauncherResultType::Calculator,
                });
            }
        } else if query.starts_with("ps") {
            results.push(LauncherResult {
                id: "ps:list".to_string(),
                name: "Process Manager".to_string(),
                description: "List running processes".to_string(),
                icon_path: Some("./assets/icons/process.svg".to_string()),
                result_type: LauncherResultType::Process,
            });
        } else {
            let apps = APPLICATIONS_SERVICE.get_all();
            let query_lower = query.to_lowercase();

            for app in apps {
                if query.is_empty()
                    || app.name.to_lowercase().contains(&query_lower)
                    || app.comment.as_ref().map(|c| c.to_lowercase().contains(&query_lower)).unwrap_or(false)
                    || app.generic_name.as_ref().map(|g| g.to_lowercase().contains(&query_lower)).unwrap_or(false) {

                    let description = app.comment
                        .or(app.generic_name)
                        .unwrap_or_else(|| "Application".to_string());

                    results.push(LauncherResult {
                        id: app.id.clone(),
                        name: app.name.clone(),
                        description,
                        icon_path: app.icon.clone(),
                        result_type: LauncherResultType::Application,
                    });

                    if results.len() >= 10 {
                        break;
                    }
                }
            }
        }

        self.set_results(cx, results);
    }

    fn update_results(&mut self, cx: &mut Cx) {
        let item_ids = [
            id!(item0), id!(item1), id!(item2), id!(item3), id!(item4),
            id!(item5), id!(item6), id!(item7), id!(item8), id!(item9)
        ];

        for (i, item_id) in item_ids.iter().enumerate() {
            let item = self.view.view(&[id!(list), *item_id]);

            if i < self.results.len() {
                let result = &self.results[i];
                item.set_visible(cx, true);

                let selected = if i == self.selected_index { 1.0 } else { 0.0 };
                item.apply_over(cx, live!{
                    draw_bg: { selected: (selected) }
                });

                item.label(&[id!(info), id!(name)]).set_text(cx, &result.name);
                item.label(&[id!(info), id!(description)]).set_text(cx, &result.description);

                if let Some(path) = &result.icon_path {
                     if std::path::Path::new(path).exists() {
                         if let Some(mut icon) = item.icon(&[id!(icon)]).borrow_mut() {
                            icon.set_icon_from_path(cx, path);
                         }
                     }
                }
            } else {
                item.set_visible(cx, false);
            }
        }
    }

    pub fn set_query(&mut self, _cx: &mut Cx, query: &str) {
        self.query = query.to_string();
        self.selected_index = 0;
    }

    pub fn set_results(&mut self, cx: &mut Cx, results: Vec<LauncherResult>) {
        self.results = results;
        self.selected_index = 0;
        self.update_results(cx);
        self.view.redraw(cx);
    }

    pub fn show(&mut self, cx: &mut Cx) {
        ::log::info!("Launcher::show() called");
        self.view.apply_over(cx, live! { visible: true });

        self.query.clear();
        self.selected_index = 0;
        self.frames_until_focus = 2;

        self.search(cx, "");

        cx.new_next_frame();
        self.view.redraw(cx);
    }

    pub fn set_text_input_focus(&mut self, cx: &mut Cx) {
        self.frames_until_focus = 2;
        self.view.redraw(cx);
    }

    pub fn hide(&mut self, cx: &mut Cx) {
        ::log::info!("Launcher::hide() called");
        self.view.apply_over(cx, live! { visible: false });
        self.query.clear();
        self.results.clear();
        self.selected_index = 0;
        self.view.text_input(ids!(search_container.search_input)).set_text(cx, "");
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
