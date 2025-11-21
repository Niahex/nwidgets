use gtk4 as gtk;
use gtk::prelude::*;
use crate::theme::colors::COLORS;
use std::cell::RefCell;
use std::rc::Rc;
use chrono::{Datelike, Duration, Local, NaiveDate, Weekday};

pub fn create_day_carousel() -> gtk::Box {
    let carousel_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    carousel_box.set_margin_start(16);
    carousel_box.set_margin_end(16);
    carousel_box.set_margin_top(8);
    carousel_box.set_margin_bottom(8);
    carousel_box.set_halign(gtk::Align::Center);

    // Style du carousel
    let carousel_css = gtk::CssProvider::new();
    carousel_css.load_from_data(&format!(
        "box {{ background-color: {}; padding: 8px; border-radius: 8px; }}",
        COLORS.polar1.to_hex_string()
    ));
    carousel_box.style_context().add_provider(
        &carousel_css,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    // Container pour les jours
    let days_container = gtk::Box::new(gtk::Orientation::Horizontal, 8);

    // État partagé : offset du carrousel (nombre de jours décalés depuis aujourd'hui)
    let carousel_offset = Rc::new(RefCell::new(0i32));

    // État partagé : index du jour sélectionné (relatif aux 7 jours visibles, 3 = milieu)
    let selected_day_index = Rc::new(RefCell::new(3usize));

    // Fonction pour reconstruire le carrousel.
    // On utilise Rc<RefCell<Option<...>>> pour permettre à la fermeture de se capturer elle-même (récursion).
    let rebuild_carousel = Rc::new(RefCell::new(None::<Box<dyn Fn()>>));
    let rebuild_carousel_clone = rebuild_carousel.clone();

    let closure = {
        let days_container = days_container.clone();
        let carousel_offset = Rc::clone(&carousel_offset);
        let selected_day_index = Rc::clone(&selected_day_index);

        move || {
            // Vider le container
            while let Some(child) = days_container.first_child() {
                days_container.remove(&child);
            }

            let today = Local::now().date_naive();
            let offset = *carousel_offset.borrow();
            let selected_idx = *selected_day_index.borrow();

            let mut day_buttons = Vec::new();

            // Créer 7 jours autour de l'offset actuel
            for i in -3..=3 {
                let date = today + Duration::days((offset + i) as i64);
                let day_num = date.day();
                let weekday = get_weekday_abbr(date.weekday());
                let visual_index = (i + 3) as usize; // 0..6
                let is_selected = visual_index == selected_idx;

                let day_button = create_day_button(day_num, &weekday, is_selected);
                day_buttons.push(day_button.clone());
                days_container.append(&day_button);
            }

            // Ajouter les handlers de clic
            let carousel_offset_clone = Rc::clone(&carousel_offset);
            let selected_day_index_clone = Rc::clone(&selected_day_index);
            
            // On clone le Rc qui contient la fermeture pour le passer aux handlers
            let rebuild_carousel_for_handlers = rebuild_carousel_clone.clone();

            for (visual_index, day_button) in day_buttons.iter().enumerate() {
                let carousel_offset_click = Rc::clone(&carousel_offset_clone);
                let selected_day_index_click = Rc::clone(&selected_day_index_clone);
                let rebuild_carousel_click = rebuild_carousel_for_handlers.clone();

                let gesture = gtk::GestureClick::new();
                gesture.connect_released(move |_, _, _, _| {
                    let _current_selected = *selected_day_index_click.borrow();

                    // Calculer la différence entre le jour cliqué et le centre (index 3)
                    let click_offset = visual_index as i32 - 3;

                    // Si on clique sur un jour qui n'est pas au centre
                    if visual_index != 3 {
                        // Décaler le carrousel pour amener ce jour au centre
                        let mut offset = carousel_offset_click.borrow_mut();
                        *offset += click_offset;
                        drop(offset);

                        // Le jour sélectionné est maintenant au centre
                        *selected_day_index_click.borrow_mut() = 3;

                        // Reconstruire le carrousel avec le nouvel offset
                        if let Some(rebuild_fn) = &*rebuild_carousel_click.borrow() {
                            rebuild_fn();
                        }

                        println!("[CAROUSEL] Shifted by {}, new offset: {}",
                                 click_offset,
                                 *carousel_offset_click.borrow());
                    } else {
                        println!("[CAROUSEL] Center day clicked (already selected)");
                    }
                });

                day_button.add_controller(gesture);
            }
        }
    };

    // On place la fermeture dans le RefCell
    *rebuild_carousel.borrow_mut() = Some(Box::new(closure));

    // Construction initiale
    if let Some(rebuild_fn) = &*rebuild_carousel.borrow() {
        rebuild_fn();
    }

    carousel_box.append(&days_container);
    carousel_box
}

fn get_weekday_abbr(weekday: Weekday) -> String {
    match weekday {
        Weekday::Mon => "Mon".to_string(),
        Weekday::Tue => "Tue".to_string(),
        Weekday::Wed => "Wed".to_string(),
        Weekday::Thu => "Thu".to_string(),
        Weekday::Fri => "Fri".to_string(),
        Weekday::Sat => "Sat".to_string(),
        Weekday::Sun => "Sun".to_string(),
    }
}

fn create_day_button(day_num: u32, weekday: &str, is_selected: bool) -> gtk::Box {
    let day_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
    day_box.set_size_request(60, 70);
    day_box.set_halign(gtk::Align::Center);
    day_box.set_valign(gtk::Align::Center);

    // Style selon si c'est le jour sélectionné ou non
    let day_css = gtk::CssProvider::new();
    if is_selected {
        day_css.load_from_data(&format!(
            "box {{
                background-color: {};
                border: 2px solid {};
                border-radius: 8px;
                padding: 8px;
            }}",
            COLORS.frost1.with_opacity(30),
            COLORS.frost1.to_hex_string()
        ));
    } else {
        day_css.load_from_data(&format!(
            "box {{
                background-color: {};
                border: 1px solid {};
                border-radius: 8px;
                padding: 8px;
            }}",
            COLORS.polar2.to_hex_string(),
            COLORS.polar3.to_hex_string()
        ));
    }
    day_box.style_context().add_provider(
        &day_css,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    // Numéro du jour
    let day_label = gtk::Label::new(Some(&day_num.to_string()));
    let num_css = gtk::CssProvider::new();
    num_css.load_from_data(&format!(
        "label {{ color: {}; font-size: 20px; font-weight: bold; }}",
        if is_selected {
            COLORS.frost1.to_hex_string()
        } else {
            COLORS.snow0.to_hex_string()
        }
    ));
    day_label.style_context().add_provider(
        &num_css,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
    day_box.append(&day_label);

    // Nom du jour (abrégé)
    let weekday_label = gtk::Label::new(Some(weekday));
    let weekday_css = gtk::CssProvider::new();
    weekday_css.load_from_data(&format!(
        "label {{ color: {}; font-size: 11px; }}",
        if is_selected {
            COLORS.frost1.to_hex_string()
        } else {
            COLORS.polar3.to_hex_string()
        }
    ));
    weekday_label.style_context().add_provider(
        &weekday_css,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
    day_box.append(&weekday_label);

    // Ajouter un cursor pointer pour indiquer que c'est cliquable
    day_box.set_cursor_from_name(Some("pointer"));

    day_box
}
