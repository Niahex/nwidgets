use gtk4 as gtk;
use gtk::prelude::*;
use std::sync::{Arc, Mutex};
use std::cell::RefCell;
use std::rc::Rc;
use crate::services::pomodoro::{PomodoroService, PomodoroState};
use crate::theme::icons;

#[derive(Clone)]
pub struct PomodoroModule {
    pub container: gtk::Box,
    icon_label: gtk::Label,
    time_label: gtk::Label,
    service: Arc<Mutex<PomodoroService>>,
}

impl PomodoroModule {
    pub fn new() -> Self {
        let container = gtk::Box::new(gtk::Orientation::Vertical, 2);
        container.set_width_request(48);
        container.set_height_request(48);
        container.set_halign(gtk::Align::Center);
        container.set_valign(gtk::Align::Center);

        let icon_label = gtk::Label::new(Some(icons::ICONS.timer_outline));
        icon_label.add_css_class("pomodoro-icon");
        icon_label.add_css_class("pomodoro-idle");

        let time_label = gtk::Label::new(Some("00:00"));
        time_label.add_css_class("pomodoro-time");

        container.append(&icon_label);
        container.append(&time_label);

        let service = Arc::new(Mutex::new(PomodoroService::new()));

        // Click gauche : toggle start/pause
        let gesture_left = gtk::GestureClick::new();
        gesture_left.set_button(1);
        let service_clone = Arc::clone(&service);
        gesture_left.connect_released(move |_, _, _, _| {
            let mut svc = service_clone.lock().unwrap();
            match svc.get_state() {
                PomodoroState::Idle => svc.start_work(),
                PomodoroState::Work | PomodoroState::ShortBreak | PomodoroState::LongBreak => svc.pause(),
                PomodoroState::WorkPaused | PomodoroState::ShortBreakPaused | PomodoroState::LongBreakPaused => svc.resume(),
            }
        });
        container.add_controller(gesture_left);

        // Click milieu : reset
        let gesture_middle = gtk::GestureClick::new();
        gesture_middle.set_button(2);
        let service_clone = Arc::clone(&service);
        gesture_middle.connect_released(move |_, _, _, _| {
            service_clone.lock().unwrap().reset();
        });
        container.add_controller(gesture_middle);

        let module = Self {
            container,
            icon_label,
            time_label,
            service,
        };

        // Auto-update toutes les secondes
        let module_clone = module.clone();
        glib::timeout_add_seconds_local(1, move || {
            module_clone.update();
            glib::ControlFlow::Continue
        });

        module.update();
        module
    }

    fn update(&self) {
        let mut svc = self.service.lock().unwrap();
        svc.auto_transition();

        let state = svc.get_state();
        let (icon, css_class) = match state {
            PomodoroState::Idle => (icons::ICONS.timer_outline, "pomodoro-idle"),
            PomodoroState::Work | PomodoroState::WorkPaused => (icons::ICONS.timer, "pomodoro-work"),
            PomodoroState::ShortBreak | PomodoroState::ShortBreakPaused => (icons::ICONS.coffee, "pomodoro-break"),
            PomodoroState::LongBreak | PomodoroState::LongBreakPaused => (icons::ICONS.beach, "pomodoro-longbreak"),
        };

        self.icon_label.set_text(icon);
        self.time_label.set_text(&svc.format_time());

        // Update CSS classes
        self.icon_label.remove_css_class("pomodoro-idle");
        self.icon_label.remove_css_class("pomodoro-work");
        self.icon_label.remove_css_class("pomodoro-break");
        self.icon_label.remove_css_class("pomodoro-longbreak");
        self.icon_label.add_css_class(css_class);
    }
}
