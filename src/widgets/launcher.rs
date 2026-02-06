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
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, 6.0);
                let color = mix(#0000, (NORD_POLAR_2), self.selected);
                sdf.fill(color);
                return sdf.result;
            }
        }

        icon = <Icon> {
            width: 24, height: 24
            icon_walk: { width: 24, height: 24 }
            draw_icon: {
                svg_file: dep("crate://self/assets/icons/svg/search.svg")
                brightness: 1.0
                curve: 0.6
                color: #fff
            }
        }

        name = <Label> {
            draw_text: { text_style: <THEME_FONT_REGULAR> { font_size: 13.0 }, color: (THEME_COLOR_TEXT_DEFAULT) }
            text: "Application"
        }
    }

    pub Launcher = {{Launcher}} {
        width: 700, height: 500

        show_bg: true
        draw_bg: {
            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, 12.0);
                sdf.fill((NORD_POLAR_0));
                
                sdf.stroke((NORD_FROST_1), 1.0);
                
                return sdf.result;
            }
        }

        flow: Down
        padding: 16
        spacing: 12

        visible: false

        search_container = <View> {
            width: Fill, height: 40
            flow: Right
            align: {x: 0.0, y: 0.5}
            padding: {left: 12, right: 12}
            spacing: 8

            show_bg: true
            draw_bg: {
                fn pixel(self) -> vec4 {
                    let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                    sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, 6.0);
                    sdf.fill((NORD_POLAR_1));
                    return sdf.result;
                }
            }

            search_icon = <Icon> {
                icon_walk: { width: 16.0 }
                draw_icon: {
                    svg_file: dep("crate://self/assets/icons/svg/search.svg")
                    color: (THEME_COLOR_TEXT_MUTE)
                }
            }

            search_input = <TextInput> {
                width: Fill, height: Fit

                draw_bg: { color: #0000 }
                draw_text: { 
                    text_style: <THEME_FONT_REGULAR> { font_size: 14.0 }, 
                    color: (THEME_COLOR_TEXT_DEFAULT) 
                }
                draw_cursor: {
                    color: (NORD_FROST_1)
                }
                empty_text: "Search apps, calculator (=), processes (ps)..."
            }
        }

        list = <List> {
            height: 400, width: Fill
            flow: Down
            spacing: 0

            clip_x: true, clip_y: true
            scroll_bars: <ScrollBars> {show_scroll_x: false, show_scroll_y: false}

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
    all_results: Vec<LauncherResult>,
    
    #[rust]
    visible_count: usize,

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
        let result = self.view.draw_walk(cx, scope, walk);
        result
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if let Event::NextFrame(_) = event {
            if self.frames_until_focus > 0 && self.view.visible {
                self.frames_until_focus -= 1;
                if self.frames_until_focus == 0 {
                    let text_input = self.view.text_input(ids!(search_input));
                    text_input.set_key_focus(cx);
                } else {
                    cx.new_next_frame();
                }
            }
        }

        if let Event::KeyDown(ke) = event {
            match ke.key_code {
                KeyCode::Escape => {
                    cx.widget_action(
                        self.widget_uid(),
                        &HeapLiveIdPath::default(),
                        LauncherAction::Close,
                    );
                    return;
                }
                KeyCode::ArrowUp => {
                    if self.selected_index > 0 {
                        self.selected_index -= 1;
                        
                        if self.selected_index >= 3 && self.all_results.len() > 10 {
                            self.load_more_results(cx);
                        }
                        
                        self.update_results(cx);
                        self.view.redraw(cx);
                    }
                    return;
                }
                KeyCode::ArrowDown => {
                    if self.selected_index < self.all_results.len().saturating_sub(1) {
                        self.selected_index += 1;
                        
                        ::log::info!("ArrowDown: selected={}, visible_count={}, all_results={}", 
                            self.selected_index, self.visible_count, self.all_results.len());
                        
                        if self.selected_index >= 7 {
                            ::log::info!("Loading more results!");
                            self.load_more_results(cx);
                        }
                        
                        self.update_results(cx);
                        self.view.redraw(cx);
                    }
                    return;
                }
                KeyCode::ReturnKey => {
                    if let Some(result) = self.results.get(self.selected_index) {
                        cx.widget_action(
                            self.widget_uid(),
                            &HeapLiveIdPath::default(),
                            LauncherAction::Launch(result.id.clone()),
                        );
                    }
                    return;
                }
                _ => {}
            }
        }

        for action in cx.capture_actions(|cx| self.view.handle_event(cx, event, scope)) {
            if let TextInputAction::Changed(new_text) = action.as_widget_action().cast() {
                self.search(cx, &new_text);
                self.update_results(cx);
            }
        }
    }
}

impl Launcher {
    fn load_more_results(&mut self, cx: &mut Cx) {
        let start_index = self.selected_index.saturating_sub(5);
        let end_index = (start_index + 10).min(self.all_results.len());
        
        ::log::info!("load_more_results: selected={}, start={}, end={}, all_results={}", 
            self.selected_index, start_index, end_index, self.all_results.len());
        
        self.results = self.all_results[start_index..end_index].to_vec();
        self.visible_count = self.results.len();
    }
    
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
                    icon_path: Some("./assets/icons/svg/search.svg".to_string()),
                    result_type: LauncherResultType::Calculator,
                });
            }
        } else if query.starts_with("ps") {
            results.push(LauncherResult {
                id: "ps:list".to_string(),
                name: "Process Manager".to_string(),
                description: "List running processes".to_string(),
                icon_path: Some("./assets/icons/svg/search.svg".to_string()),
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
            }
        }
    }

    self.all_results = results;
    self.visible_count = 10.min(self.all_results.len());
    self.results = self.all_results[..self.visible_count].to_vec();
    ::log::info!("search complete: all_results={}, visible_count={}", 
        self.all_results.len(), self.visible_count);
    self.update_results(cx);
}

    fn update_results(&mut self, cx: &mut Cx) {
        let item_ids = [
            id!(item0), id!(item1), id!(item2), id!(item3), id!(item4),
            id!(item5), id!(item6), id!(item7), id!(item8), id!(item9)
        ];
        
        let start_index = self.selected_index.saturating_sub(5);
        let display_offset = self.selected_index - start_index;

        for (i, item_id) in item_ids.iter().enumerate() {
            let item = self.view.view(&[id!(list), *item_id]);
            let result_index = start_index + i;

            if result_index < self.all_results.len() {
                let result = &self.all_results[result_index];
                item.set_visible(cx, true);

                let selected = if i == display_offset { 1.0 } else { 0.0 };
                item.apply_over(cx, live!{
                    draw_bg: { selected: (selected) }
                });

                item.label(&[id!(name)]).set_text(cx, &result.name);

                if let Some(path) = &result.icon_path {
                    if std::path::Path::new(path).exists() {
                        if let Some(mut icon) = item.icon(&[id!(icon)]).borrow_mut() {
                        }
                    } else {
                        if let Some(mut icon) = item.icon(&[id!(icon)]).borrow_mut() {
                        }
                    }
                } else {
                    if let Some(mut icon) = item.icon(&[id!(icon)]).borrow_mut() {
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
        self.view.apply_over(cx, live! { visible: true });
        self.search(cx, "");

        cx.widget_action(self.view.widget_uid(), &Scope::empty().path, LauncherAction::Shown);
        cx.new_next_frame();
        self.view.redraw(cx);
    }

    pub fn hide(&mut self, cx: &mut Cx) {
        self.view.apply_over(cx, live! { visible: false });
        self.query.clear();
        self.results.clear();
        self.all_results.clear();
        self.visible_count = 0;
        self.selected_index = 0;
        self.view.text_input(ids!(search_container.search_input)).set_text(cx, "");
        
        cx.widget_action(self.view.widget_uid(), &Scope::empty().path, LauncherAction::Hidden);
        self.view.redraw(cx);
    }

    pub fn set_text_input_focus(&mut self, cx: &mut Cx) {
        self.frames_until_focus = 2;
        self.view.redraw(cx);
    }
}

#[derive(Clone, Debug, DefaultNone)]
pub enum LauncherAction {
    None,
    Close,
    Launch(String),
    QueryChanged(String),
    Shown,
    Hidden,
}
