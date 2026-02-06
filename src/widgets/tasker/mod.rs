use makepad_widgets::*;
use chrono::{NaiveDate, Datelike};
use crate::services::tasker::Task;
use crate::TASKER_SERVICE;
use std::collections::HashMap;

live_design! {
    use link::theme::*;
    use link::widgets::*;
    use makepad_draw::shader::std::*;
    use crate::theme::*;

    DateItem = <Button> {
        width: 100, height: 80
        flow: Down
        align: {x: 0.5, y: 0.5}
        spacing: 4
        padding: 8

        draw_bg: {
            instance selected: 0.0
            instance is_today: 0.0

            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, 8.0);
                
                let bg_color = mix(
                    (NORD_POLAR_1),
                    (NORD_FROST_1),
                    self.selected
                );
                
                sdf.fill(bg_color);
                
                if self.is_today > 0.5 {
                    sdf.stroke((NORD_AURORA_ORANGE), 2.0);
                }
                
                return sdf.result;
            }
        }
        
        draw_text: { visible: false }

        content = <View> {
            width: Fill, height: Fill
            flow: Down
            align: {x: 0.5, y: 0.5}
            spacing: 4

            day_name = <Label> {
                draw_text: { 
                    text_style: <THEME_FONT_REGULAR> { font_size: 11.0 }, 
                    color: (THEME_COLOR_TEXT_MUTE) 
                }
                text: "Mon"
            }

            day_number = <Label> {
                draw_text: { 
                    text_style: <THEME_FONT_BOLD> { font_size: 20.0 }, 
                    color: (THEME_COLOR_TEXT_DEFAULT) 
                }
                text: "15"
            }

            task_count = <Label> {
                draw_text: { 
                    text_style: <THEME_FONT_REGULAR> { font_size: 10.0 }, 
                    color: (THEME_COLOR_TEXT_MUTE) 
                }
                text: "3 tasks"
            }
        }
    }

    TaskItem = <View> {
        width: Fill, height: Fit
        flow: Right
        align: {x: 0.0, y: 0.5}
        padding: {top: 8, bottom: 8, left: 12, right: 12}
        spacing: 12

        show_bg: true
        draw_bg: {
            instance hover: 0.0

            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, 6.0);
                let color = mix(#0000, (NORD_POLAR_2), self.hover);
     sdf.fill(color);
                return sdf.result;
            }
        }

        checkbox = <Button> {
            width: 20, height: 20
            draw_bg: {
                instance checked: 0.0

                fn pixel(self) -> vec4 {
                    let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                    sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, 4.0);
                    
                    let border_color = mix((NORD_POLAR_3), (NORD_FROST_1), self.checked);
                    sdf.stroke(border_color, 2.0);
                    
                    if self.checked > 0.5 {
                        sdf.move_to(4.0, 10.0);
                        sdf.line_to(8.0, 14.0);
                        sdf.line_to(16.0, 6.0);
                        sdf.stroke((NORD_FROST_1), 2.0);
                    }
                    
                    return sdf.result;
                }
            }
            draw_text: { visible: false }
        }

        task_name = <Label> {
            draw_text: { 
                text_style: <THEME_FONT_REGULAR> { font_size: 13.0 }, 
                color: (THEME_COLOR_TEXT_DEFAULT) 
            }
            text: "Task name"
        }
    }

    pub Tasker = {{Tasker}} {
        width: 800, height: 600

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

        header = <View> {
            width: Fill, height: Fit
            flow: Right
            align: {x: 0.0, y: 0.5}
            spacing: 12

            title = <Label> {
                draw_text: { 
                    text_style: <THEME_FONT_BOLD> { font_size: 18.0 }, 
                    color: (THEME_COLOR_TEXT_DEFAULT) 
                }
                text: "Tasker"
            }
        }

        date_carousel = <View> {
            width: Fill, height: 100
            flow: Right
            align: {x: 0.5, y: 0.5}
            spacing: 8
            padding: {top: 8, bottom: 8}

            dates_container = <View> {
                width: Fit, height: Fit
                flow: Right
                spacing: 8

                date0 = <DateItem> {visible: false}
                date1 = <DateItem> {visible: false}
                date2 = <DateItem> {visible: false}
                date3 = <DateItem> {visible: false}
                date4 = <DateItem> {visible: false}
            }
        }

        content = <View> {
            width: Fill, height: Fill
            flow: Down
            spacing: 8

            add_task_container = <View> {
                width: Fill, height: Fit
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

                task_input = <TextInput> {
                    width: Fill, height: 40

                    draw_bg: { color: #0000 }
                    draw_text: { 
                        text_style: <THEME_FONT_REGULAR> { font_size: 13.0 }, 
                        color: (THEME_COLOR_TEXT_DEFAULT) 
                    }
                    draw_cursor: {
                        color: (NORD_FROST_1)
                    }
                    empty_text: "Add a new task..."
                }

                add_button = <Button> {
                    width: 60, height: 32
                    draw_bg: {
                        fn pixel(self) -> vec4 {
                            let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                            sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, 4.0);
                            sdf.fill((NORD_FROST_1));
                            return sdf.result;
                        }
                    }
                    draw_text: { 
                        text_style: <THEME_FONT_BOLD> { font_size: 12.0 }, 
                        color: (NORD_POLAR_0) 
                    }
                    text: "Add"
                }
            }

            task_list = <View> {
                width: Fill, height: Fill
                flow: Down
                spacing: 4
                padding: {top: 8}

                clip_x: true, clip_y: true
                scroll_bars: <ScrollBars> {show_scroll_x: false, show_scroll_y: true}

                task0 = <TaskItem> {visible: false}
                task1 = <TaskItem> {visible: false}
                task2 = <TaskItem> {visible: false}
                task3 = <TaskItem> {visible: false}
                task4 = <TaskItem> {visible: false}
                task5 = <TaskItem> {visible: false}
                task6 = <TaskItem> {visible: false}
                task7 = <TaskItem> {visible: false}
                task8 = <TaskItem> {visible: false}
                task9 = <TaskItem> {visible: false}
                task10 = <TaskItem> {visible: false}
                task11 = <TaskItem> {visible: false}
                task12 = <TaskItem> {visible: false}
                task13 = <TaskItem> {visible: false}
                task14 = <TaskItem> {visible: false}
            }
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct Tasker {
    #[deref]
    view: View,

    #[rust]
    current_date: NaiveDate,

    #[rust]
    selected_date: NaiveDate,

    #[rust]
    tasks_for_selected_date: Vec<Task>,

    #[rust]
    uid_to_task: HashMap<WidgetUid, String>,

    #[rust]
    uid_to_date: HashMap<WidgetUid, NaiveDate>,
}

impl Widget for Tasker {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if let Event::KeyDown(ke) = event {
            match ke.key_code {
                KeyCode::Escape => {
                    self.hide(cx);
                    return;
                }
                KeyCode::ArrowLeft => {
                    self.selected_date = self.selected_date.checked_sub_signed(chrono::Duration::days(1)).unwrap_or(self.selected_date);
                    self.load_tasks_for_selected_date();
                    self.render_carousel(cx);
                    self.render_tasks(cx);
                    return;
                }
                KeyCode::ArrowRight => {
                    self.selected_date = self.selected_date.checked_add_signed(chrono::Duration::days(1)).unwrap_or(self.selected_date);
                    self.load_tasks_for_selected_date();
                    self.render_carousel(cx);
                    self.render_tasks(cx);
                    return;
                }
                _ => {}
            }
        }

        let actions = cx.capture_actions(|cx| self.view.handle_event(cx, event, scope));
        
        if self.view.button(ids!(content.add_task_container.add_button)).clicked(&actions) {
            let task_name = self.view.text_input(ids!(content.add_task_container.task_input)).text();
            if !task_name.is_empty() {
                TASKER_SERVICE.add_task(self.selected_date, task_name);
                self.view.text_input(ids!(content.add_task_container.task_input)).set_text(cx, "");
                self.load_tasks_for_selected_date();
                self.render_tasks(cx);
                self.render_carousel(cx);
            }
        }

        if self.view.text_input(ids!(content.add_task_container.task_input)).returned(&actions).is_some() {
            let task_name = self.view.text_input(ids!(content.add_task_container.task_input)).text();
            if !task_name.is_empty() {
                TASKER_SERVICE.add_task(self.selected_date, task_name);
                self.view.text_input(ids!(content.add_task_container.task_input)).set_text(cx, "");
                self.load_tasks_for_selected_date();
                self.render_tasks(cx);
                self.render_carousel(cx);
            }
        }

        for i in 0..5 {
            let date_id = LiveId::from_str(&format!("date{}", i));
            if self.view.view(ids!(date_carousel.dates_container)).button(&[date_id]).clicked(&actions) {
                let date_opt = self.uid_to_date.get(&self.view.view(ids!(date_carousel.dates_container)).button(&[date_id]).widget_uid()).cloned();
                if let Some(date) = date_opt {
                    self.selected_date = date;
                    self.load_tasks_for_selected_date();
                    self.render_carousel(cx);
                    self.render_tasks(cx);
                }
            }
        }

        for i in 0..15 {
            let task_id = LiveId::from_str(&format!("task{}", i));
            let task_view = self.view.view(ids!(content.task_list)).view(&[task_id]);
            if task_view.button(ids!(checkbox)).clicked(&actions) {
                let task_id_opt = self.uid_to_task.get(&task_view.button(ids!(checkbox)).widget_uid()).cloned();
                if let Some(task_id) = task_id_opt {
                    TASKER_SERVICE.toggle_task(&task_id);
                    self.load_tasks_for_selected_date();
                    self.render_tasks(cx);
                    self.render_carousel(cx);
                    
                    cx.widget_action(self.view.widget_uid(), &Scope::empty().path, TaskerAction::TaskToggled(task_id));
                }
            }
        }
    }
}

impl Tasker {
    pub fn show(&mut self, cx: &mut Cx) {
        self.view.apply_over(cx, live! { visible: true });
        
        self.current_date = crate::services::tasker::TaskerService::get_today();
        self.selected_date = self.current_date;
        
        self.load_tasks_for_selected_date();
        self.render_carousel(cx);
        self.render_tasks(cx);

        cx.widget_action(self.view.widget_uid(), &Scope::empty().path, TaskerAction::Shown);
        cx.new_next_frame();
        self.view.redraw(cx);
    }

    pub fn hide(&mut self, cx: &mut Cx) {
        self.view.apply_over(cx, live! { visible: false });
        
        cx.widget_action(self.view.widget_uid(), &Scope::empty().path, TaskerAction::Hidden);
        self.view.redraw(cx);
    }

    fn load_tasks_for_selected_date(&mut self) {
        self.tasks_for_selected_date = TASKER_SERVICE.get_tasks_for_date(self.selected_date);
    }

    fn render_carousel(&mut self, cx: &mut Cx) {
        self.uid_to_date.clear();

        for i in 0..5 {
            let offset = i as i64 - 2;
            let date = self.selected_date.checked_add_signed(chrono::Duration::days(offset)).unwrap_or(self.selected_date);
            let date_id = LiveId::from_str(&format!("date{}", i));
            let date_item = self.view.view(ids!(date_carousel.dates_container)).button(&[date_id]);
            
            date_item.set_visible(cx, true);

            let day_name = match date.weekday() {
                chrono::Weekday::Mon => "Mon",
                chrono::Weekday::Tue => "Tue",
                chrono::Weekday::Wed => "Wed",
                chrono::Weekday::Thu => "Thu",
                chrono::Weekday::Fri => "Fri",
                chrono::Weekday::Sat => "Sat",
                chrono::Weekday::Sun => "Sun",
            };

            let uid = date_item.widget_uid();
            
            date_item.label(ids!(content.day_name)).set_text(cx, day_name);
            date_item.label(ids!(content.day_number)).set_text(cx, &format!("{}", date.day()));

            let tasks = TASKER_SERVICE.get_tasks_for_date(date);
            let task_count = tasks.len();
            let completed_count = tasks.iter().filter(|t| t.completed).count();
            date_item.label(ids!(content.task_count)).set_text(cx, &format!("{}/{}", completed_count, task_count));

            let is_selected = i == 2;
            let is_today = date == self.current_date;
            
            date_item.apply_over(cx, live! {
                draw_bg: {
                    selected: (if is_selected { 1.0 } else { 0.0 })
                    is_today: (if is_today { 1.0 } else { 0.0 })
                }
            });

            self.uid_to_date.insert(uid, date);
        }

        self.view.redraw(cx);
    }

    fn render_tasks(&mut self, cx: &mut Cx) {
        self.uid_to_task.clear();

        for i in 0..15 {
            let task_id = LiveId::from_str(&format!("task{}", i));
            let task_item = self.view.view(ids!(content.task_list)).view(&[task_id]);
            
            if i < self.tasks_for_selected_date.len() {
                let task = &self.tasks_for_selected_date[i];
                task_item.set_visible(cx, true);
                
                let checkbox = task_item.button(ids!(checkbox));
                let checkbox_uid = checkbox.widget_uid();
                
                task_item.label(ids!(task_name)).set_text(cx, &task.name);
                
                let checked = if task.completed { 1.0 } else { 0.0 };
                checkbox.apply_over(cx, live! { draw_bg: { checked: (checked) } });

                self.uid_to_task.insert(checkbox_uid, task.id.clone());
            } else {
                task_item.set_visible(cx, false);
            }
        }

        self.view.redraw(cx);
    }
}

#[derive(Clone, Debug, DefaultNone)]
pub enum TaskerAction {
    None,
    Close,
    Shown,
    Hidden,
    DateSelected(NaiveDate),
    TaskToggled(String),
}
