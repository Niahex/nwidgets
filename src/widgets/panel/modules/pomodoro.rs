use gtk4 as gtk;
use gtk::prelude::*;
use std::sync::{Arc, Mutex};
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
        container.add_css_class("pomodoro-widget");
        container.set_width_request(48);
        container.set_height_request(48);
        container.set_halign(gtk::Align::Center);
        container.set_valign(gtk::Align::Center);

        let icon_label = gtk::Label::new(Some(icons::ICONS.play));
        icon_label.add_css_class("pomodoro-icon");
        icon_label.add_css_class("pomodoro-idle");

        let time_label = gtk::Label::new(Some("00:00"));
        time_label.add_css_class("pomodoro-time");

        container.append(&icon_label);
        container.append(&time_label);

        let service = Arc::new(Mutex::new(PomodoroService::new()));

        let module = Self {
            container: container.clone(),
            icon_label: icon_label.clone(),
            time_label: time_label.clone(),
            service: service.clone(),
        };

        // Click gauche : toggle start/pause
        let gesture_left = gtk::GestureClick::new();
        gesture_left.set_button(1);
        let service_clone = Arc::clone(&service);
        let module_clone = module.clone();
        gesture_left.connect_released(move |_, _, _, _| {
            let mut svc = service_clone.lock().unwrap();
            match svc.get_state() {
                PomodoroState::Idle => svc.start_work(),
                PomodoroState::Work | PomodoroState::ShortBreak | PomodoroState::LongBreak => svc.pause(),
                PomodoroState::WorkPaused | PomodoroState::ShortBreakPaused | PomodoroState::LongBreakPaused => svc.resume(),
            }
            drop(svc);
            module_clone.update();
        });
        container.add_controller(gesture_left);

        // Click milieu : reset
        let gesture_middle = gtk::GestureClick::new();
        gesture_middle.set_button(2);
        let service_clone = Arc::clone(&service);
        let module_clone = module.clone();
        gesture_middle.connect_released(move |_, _, _, _| {
            service_clone.lock().unwrap().reset();
            module_clone.update();
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
        let formatted_time = svc.format_time();
        let (_icon, time_text, css_class) = match state {
            PomodoroState::Idle => ("", "00:00", "pomodoro-idle"),
            PomodoroState::Work => ("", formatted_time.as_str(), "pomodoro-work"),
            PomodoroState::WorkPaused => ("", formatted_time.as_str(), "pomodoro-work"),
            PomodoroState::ShortBreak => ("", formatted_time.as_str(), "pomodoro-break"),
            PomodoroState::ShortBreakPaused => ("", formatted_time.as_str(), "pomodoro-break"),
            PomodoroState::LongBreak => ("", formatted_time.as_str(), "pomodoro-longbreak"),
            PomodoroState::LongBreakPaused => ("", formatted_time.as_str(), "pomodoro-longbreak"),
        };

        // Show icon only when idle or paused
        if state == PomodoroState::Idle {
            self.icon_label.set_text(icons::ICONS.play);
            self.icon_label.set_visible(true);
            self.time_label.set_visible(false);
        } else if matches!(state, PomodoroState::WorkPaused | PomodoroState::ShortBreakPaused | PomodoroState::LongBreakPaused) {
            self.icon_label.set_text(icons::ICONS.pause);
            self.icon_label.set_visible(true);
            self.time_label.set_visible(false);
        } else {
            self.icon_label.set_visible(false);
            self.time_label.set_text(time_text);
            self.time_label.set_visible(true);
        }

        // Update CSS classes
        self.time_label.remove_css_class("pomodoro-idle");
        self.time_label.remove_css_class("pomodoro-work");
        self.time_label.remove_css_class("pomodoro-break");
        self.time_label.remove_css_class("pomodoro-longbreak");
        self.time_label.add_css_class(css_class);
    }
}
