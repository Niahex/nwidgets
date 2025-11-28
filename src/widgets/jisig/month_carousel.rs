use gtk4 as gtk;
use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use chrono::{Datelike, Local, NaiveDate};

pub fn create_month_carousel() -> (gtk::Box, Rc<RefCell<Option<Box<dyn Fn(NaiveDate)>>>>, Rc<RefCell<Option<Box<dyn Fn()>>>>) {
    let carousel_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    carousel_box.set_margin_start(16);
    carousel_box.set_margin_end(16);
    carousel_box.set_margin_top(8);
    carousel_box.set_margin_bottom(8);
    carousel_box.set_halign(gtk::Align::Center);
    carousel_box.add_css_class("month-carousel");

    let months_container = gtk::Box::new(gtk::Orientation::Horizontal, 8);

    // Offset en mois depuis le mois actuel
    let month_offset = Rc::new(RefCell::new(0i32));
    let selected_month_index = Rc::new(RefCell::new(4usize)); // Milieu des 9 mois

    let on_date_changed: Rc<RefCell<Option<Box<dyn Fn(NaiveDate)>>>> = Rc::new(RefCell::new(None));

    let rebuild_carousel = Rc::new(RefCell::new(None::<Box<dyn Fn()>>));
    let rebuild_carousel_clone = rebuild_carousel.clone();

    let closure = {
        let months_container = months_container.clone();
        let month_offset = Rc::clone(&month_offset);
        let selected_month_index = Rc::clone(&selected_month_index);
        let on_date_changed = Rc::clone(&on_date_changed);

        move || {
            while let Some(child) = months_container.first_child() {
                months_container.remove(&child);
            }

            let today = Local::now().date_naive();
            let offset = *month_offset.borrow();
            let selected_idx = *selected_month_index.borrow();

            let mut month_buttons = Vec::new();

            // Créer 9 mois
            for i in -4..=4 {
                let month_date = get_month_start(today, offset + i);
                let visual_index = (i + 4) as usize; // 0..8
                let is_selected = visual_index == selected_idx;

                let month_button = create_month_button(&month_date, is_selected);
                month_buttons.push(month_button.clone());
                months_container.append(&month_button);
            }

            // Notifier le changement de date
            let selected_date = get_month_start(today, offset);
            if let Some(callback) = &*on_date_changed.borrow() {
                callback(selected_date);
            }

            // Ajouter les handlers de clic
            let month_offset_clone = Rc::clone(&month_offset);
            let selected_month_index_clone = Rc::clone(&selected_month_index);
            let rebuild_carousel_for_handlers = rebuild_carousel_clone.clone();

            for (visual_index, month_button) in month_buttons.iter().enumerate() {
                let month_offset_click = Rc::clone(&month_offset_clone);
                let selected_month_index_click = Rc::clone(&selected_month_index_clone);
                let rebuild_carousel_click = rebuild_carousel_for_handlers.clone();

                let gesture = gtk::GestureClick::new();
                gesture.connect_released(move |_, _, _, _| {
                    let click_offset = visual_index as i32 - 4;

                    if visual_index != 4 {
                        let mut offset = month_offset_click.borrow_mut();
                        *offset += click_offset;
                        drop(offset);

                        *selected_month_index_click.borrow_mut() = 4;

                        if let Some(rebuild_fn) = &*rebuild_carousel_click.borrow() {
                            rebuild_fn();
                        }
                    }
                });

                month_button.add_controller(gesture);
            }
        }
    };

    *rebuild_carousel.borrow_mut() = Some(Box::new(closure));

    if let Some(rebuild_fn) = &*rebuild_carousel.borrow() {
        rebuild_fn();
    }

    carousel_box.append(&months_container);

    // Callback pour réinitialiser
    let reset_to_today: Rc<RefCell<Option<Box<dyn Fn()>>>> = Rc::new(RefCell::new(None));
    let reset_closure = {
        let month_offset = Rc::clone(&month_offset);
        let selected_month_index = Rc::clone(&selected_month_index);
        let rebuild_carousel = Rc::clone(&rebuild_carousel);

        Box::new(move || {
            *month_offset.borrow_mut() = 0;
            *selected_month_index.borrow_mut() = 4;
            if let Some(rebuild_fn) = &*rebuild_carousel.borrow() {
                rebuild_fn();
            }
        })
    };
    *reset_to_today.borrow_mut() = Some(reset_closure);

    (carousel_box, on_date_changed, reset_to_today)
}

fn get_month_start(date: NaiveDate, month_offset: i32) -> NaiveDate {
    let mut year = date.year();
    let mut month = date.month() as i32 + month_offset;

    while month > 12 {
        month -= 12;
        year += 1;
    }

    while month < 1 {
        month += 12;
        year -= 1;
    }

    NaiveDate::from_ymd_opt(year, month as u32, 1).unwrap()
}

fn create_month_button(month_date: &NaiveDate, is_selected: bool) -> gtk::Box {
    let month_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
    month_box.set_size_request(100, 70);
    month_box.set_halign(gtk::Align::Center);
    month_box.set_valign(gtk::Align::Center);
    month_box.set_cursor_from_name(Some("pointer"));

    if is_selected {
        month_box.add_css_class("month-button-selected");
    } else {
        month_box.add_css_class("month-button");
    }

    // Mois (abrégé)
    let month_name = month_date.format("%b").to_string();
    let month_label = gtk::Label::new(Some(&month_name));
    if is_selected {
        month_label.add_css_class("month-name-selected");
    } else {
        month_label.add_css_class("month-name");
    }
    month_box.append(&month_label);

    // Année (2 chiffres)
    let year_text = month_date.format("%y").to_string();
    let year_label = gtk::Label::new(Some(&year_text));
    if is_selected {
        year_label.add_css_class("month-year-selected");
    } else {
        year_label.add_css_class("month-year");
    }
    month_box.append(&year_label);

    month_box
}
