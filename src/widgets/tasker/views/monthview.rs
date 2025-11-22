use gtk4 as gtk;
use gtk::prelude::*;
use crate::theme::colors::COLORS;
use chrono::{Datelike, Local, NaiveDate};

pub fn create_monthview() -> gtk::ScrolledWindow {
    let scrolled = gtk::ScrolledWindow::new();
    scrolled.set_vexpand(true);
    scrolled.set_hscrollbar_policy(gtk::PolicyType::Never);
    scrolled.set_vscrollbar_policy(gtk::PolicyType::Automatic);

    let month_box = gtk::Box::new(gtk::Orientation::Vertical, 8);
    month_box.set_margin_start(16);
    month_box.set_margin_end(16);
    month_box.set_margin_top(16);
    month_box.set_margin_bottom(16);

    let bg_css = gtk::CssProvider::new();
    bg_css.load_from_data(&format!(
        "box {{ background-color: {}; }}",
        COLORS.polar0.to_hex_string()
    ));
    month_box.style_context().add_provider(
        &bg_css,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    // En-tête des jours de la semaine
    let days_header = create_days_header();
    month_box.append(&days_header);

    // Grille du calendrier
    let calendar_grid = create_calendar_grid();
    month_box.append(&calendar_grid);

    scrolled.set_child(Some(&month_box));
    scrolled
}

fn create_days_header() -> gtk::Box {
    let header = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    header.set_homogeneous(true);

    let days = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

    for day in days.iter() {
        let label = gtk::Label::new(Some(day));
        let css = gtk::CssProvider::new();
        css.load_from_data(&format!(
            "label {{
                color: {};
                font-weight: bold;
                padding: 8px;
            }}",
            COLORS.frost1.to_hex_string()
        ));
        label.style_context().add_provider(
            &css,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
        header.append(&label);
    }

    header
}

fn create_calendar_grid() -> gtk::Grid {
    let grid = gtk::Grid::new();
    grid.set_row_homogeneous(true);
    grid.set_column_homogeneous(true);
    grid.set_row_spacing(4);
    grid.set_column_spacing(4);

    let today = Local::now().date_naive();
    let first_of_month = NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap();
    let days_in_month = get_days_in_month(today.year(), today.month());

    // Calculer le jour de la semaine du premier jour (0 = Lundi, 6 = Dimanche)
    let start_weekday = first_of_month.weekday().num_days_from_monday() as i32;

    for day in 1..=days_in_month {
        let day_cell = create_day_cell(day, day == today.day());
        let col = ((start_weekday + day as i32 - 1) % 7) as i32;
        let row = ((start_weekday + day as i32 - 1) / 7) as i32;
        grid.attach(&day_cell, col, row, 1, 1);
    }

    grid
}

fn create_day_cell(day: u32, is_today: bool) -> gtk::Box {
    let cell = gtk::Box::new(gtk::Orientation::Vertical, 4);
    cell.set_size_request(120, 90);

    let css = gtk::CssProvider::new();
    let (bg_color, border) = if is_today {
        (
            COLORS.frost1.with_opacity(30),
            format!("border: 2px solid {};", COLORS.frost1.to_hex_string())
        )
    } else {
        (COLORS.polar2.to_hex_string(), String::from("border: 1px solid ") + &COLORS.polar3.to_hex_string() + ";")
    };

    css.load_from_data(&format!(
        "box {{
            background-color: {};
            {}
            border-radius: 8px;
            padding: 8px;
        }}",
        bg_color, border
    ));
    cell.style_context().add_provider(
        &css,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let day_label = gtk::Label::new(Some(&day.to_string()));
    let label_css = gtk::CssProvider::new();
    label_css.load_from_data(&format!(
        "label {{
            color: {};
            font-weight: bold;
        }}",
        if is_today {
            COLORS.frost1.to_hex_string()
        } else {
            COLORS.snow0.to_hex_string()
        }
    ));
    day_label.style_context().add_provider(
        &label_css,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
    day_label.set_halign(gtk::Align::Start);
    cell.append(&day_label);

    cell
}

fn get_days_in_month(year: i32, month: u32) -> u32 {
    let next_month = if month == 12 { 1 } else { month + 1 };
    let next_year = if month == 12 { year + 1 } else { year };

    NaiveDate::from_ymd_opt(next_year, next_month, 1)
        .unwrap()
        .signed_duration_since(NaiveDate::from_ymd_opt(year, month, 1).unwrap())
        .num_days() as u32
}
