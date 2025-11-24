use gtk4 as gtk;
use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use chrono::{Datelike, Duration, Local, NaiveDate};

pub fn create_week_carousel() -> (gtk::Box, Rc<RefCell<Option<Box<dyn Fn(NaiveDate)>>>>, Rc<RefCell<Option<Box<dyn Fn()>>>>) {
    let carousel_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    carousel_box.set_margin_start(16);
    carousel_box.set_margin_end(16);
    carousel_box.set_margin_top(8);
    carousel_box.set_margin_bottom(8);
    carousel_box.set_halign(gtk::Align::Center);
    carousel_box.add_css_class("week-carousel");

    let weeks_container = gtk::Box::new(gtk::Orientation::Horizontal, 8);

    // Offset en semaines depuis la semaine actuelle
    let week_offset = Rc::new(RefCell::new(0i32));
    let selected_week_index = Rc::new(RefCell::new(4usize)); // Milieu des 9 semaines

    let on_date_changed: Rc<RefCell<Option<Box<dyn Fn(NaiveDate)>>>> = Rc::new(RefCell::new(None));

    let rebuild_carousel = Rc::new(RefCell::new(None::<Box<dyn Fn()>>));
    let rebuild_carousel_clone = rebuild_carousel.clone();

    let closure = {
        let weeks_container = weeks_container.clone();
        let week_offset = Rc::clone(&week_offset);
        let selected_week_index = Rc::clone(&selected_week_index);
        let on_date_changed = Rc::clone(&on_date_changed);

        move || {
            while let Some(child) = weeks_container.first_child() {
                weeks_container.remove(&child);
            }

            let today = Local::now().date_naive();
            let offset = *week_offset.borrow();
            let selected_idx = *selected_week_index.borrow();

            let mut week_buttons = Vec::new();

            // Créer 9 semaines
            for i in -4..=4 {
                let week_start = get_week_start(today, offset + i);
                let week_end = week_start + Duration::days(6);
                let visual_index = (i + 4) as usize; // 0..8
                let is_selected = visual_index == selected_idx;

                let week_button = create_week_button(&week_start, &week_end, is_selected);
                week_buttons.push(week_button.clone());
                weeks_container.append(&week_button);
            }

            // Notifier le changement de date
            let selected_date = get_week_start(today, offset);
            if let Some(callback) = &*on_date_changed.borrow() {
                callback(selected_date);
            }

            // Ajouter les handlers de clic
            let week_offset_clone = Rc::clone(&week_offset);
            let selected_week_index_clone = Rc::clone(&selected_week_index);
            let rebuild_carousel_for_handlers = rebuild_carousel_clone.clone();

            for (visual_index, week_button) in week_buttons.iter().enumerate() {
                let week_offset_click = Rc::clone(&week_offset_clone);
                let selected_week_index_click = Rc::clone(&selected_week_index_clone);
                let rebuild_carousel_click = rebuild_carousel_for_handlers.clone();

                let gesture = gtk::GestureClick::new();
                gesture.connect_released(move |_, _, _, _| {
                    let click_offset = visual_index as i32 - 4;

                    if visual_index != 4 {
                        let mut offset = week_offset_click.borrow_mut();
                        *offset += click_offset;
                        drop(offset);

                        *selected_week_index_click.borrow_mut() = 4;

                        if let Some(rebuild_fn) = &*rebuild_carousel_click.borrow() {
                            rebuild_fn();
                        }
                    }
                });

                week_button.add_controller(gesture);
            }
        }
    };

    *rebuild_carousel.borrow_mut() = Some(Box::new(closure));

    if let Some(rebuild_fn) = &*rebuild_carousel.borrow() {
        rebuild_fn();
    }

    carousel_box.append(&weeks_container);

    // Callback pour réinitialiser
    let reset_to_today: Rc<RefCell<Option<Box<dyn Fn()>>>> = Rc::new(RefCell::new(None));
    let reset_closure = {
        let week_offset = Rc::clone(&week_offset);
        let selected_week_index = Rc::clone(&selected_week_index);
        let rebuild_carousel = Rc::clone(&rebuild_carousel);

        Box::new(move || {
            *week_offset.borrow_mut() = 0;
            *selected_week_index.borrow_mut() = 4;
            if let Some(rebuild_fn) = &*rebuild_carousel.borrow() {
                rebuild_fn();
            }
        })
    };
    *reset_to_today.borrow_mut() = Some(reset_closure);

    (carousel_box, on_date_changed, reset_to_today)
}

fn get_week_start(date: NaiveDate, week_offset: i32) -> NaiveDate {
    let target_date = date + Duration::weeks(week_offset as i64);
    let weekday = target_date.weekday().num_days_from_monday() as i64;
    target_date - Duration::days(weekday)
}

fn create_week_button(week_start: &NaiveDate, week_end: &NaiveDate, is_selected: bool) -> gtk::Box {
    let week_box = gtk::Box::new(gtk::Orientation::Vertical, 2);
    week_box.set_size_request(80, 80);
    week_box.set_halign(gtk::Align::Center);
    week_box.set_valign(gtk::Align::Center);
    week_box.set_cursor_from_name(Some("pointer"));

    if is_selected {
        week_box.add_css_class("week-button-selected");
    } else {
        week_box.add_css_class("week-button");
    }

    // Jour de début
    let start_day = week_start.day();
    let start_label = gtk::Label::new(Some(&start_day.to_string()));
    if is_selected {
        start_label.add_css_class("week-day-selected");
    } else {
        start_label.add_css_class("week-day");
    }
    week_box.append(&start_label);

    // Jour de fin
    let end_day = week_end.day();
    let end_label = gtk::Label::new(Some(&end_day.to_string()));
    if is_selected {
        end_label.add_css_class("week-day-selected");
    } else {
        end_label.add_css_class("week-day");
    }
    week_box.append(&end_label);

    // Mois et année
    let month_year = format!("{} - {}",
        week_start.format("%b").to_string().to_lowercase(),
        week_start.format("%y")
    );
    let month_label = gtk::Label::new(Some(&month_year));
    if is_selected {
        month_label.add_css_class("week-month-selected");
    } else {
        month_label.add_css_class("week-month");
    }
    week_box.append(&month_label);

    week_box
}
