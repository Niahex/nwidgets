use gtk4 as gtk;
use gtk::prelude::*;
use crate::theme::colors::COLORS;

pub fn create_weekview() -> gtk::ScrolledWindow {
    let scrolled = gtk::ScrolledWindow::new();
    scrolled.set_vexpand(true);
    scrolled.set_hscrollbar_policy(gtk::PolicyType::Never);
    scrolled.set_vscrollbar_policy(gtk::PolicyType::Automatic);

    let week_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    week_box.set_margin_start(16);
    week_box.set_margin_end(16);
    week_box.set_margin_top(16);
    week_box.set_margin_bottom(16);
    week_box.set_homogeneous(true);

    let bg_css = gtk::CssProvider::new();
    bg_css.load_from_data(&format!(
        "box {{ background-color: {}; }}",
        COLORS.polar0.to_hex_string()
    ));
    week_box.style_context().add_provider(
        &bg_css,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let days = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

    for day in days.iter() {
        let day_column = create_day_column(day);
        week_box.append(&day_column);
    }

    scrolled.set_child(Some(&week_box));
    scrolled
}

fn create_day_column(day_name: &str) -> gtk::Box {
    let column = gtk::Box::new(gtk::Orientation::Vertical, 8);
    column.set_margin_start(4);
    column.set_margin_end(4);

    let day_label = gtk::Label::new(Some(day_name));
    let label_css = gtk::CssProvider::new();
    label_css.load_from_data(&format!(
        "label {{
            color: {};
            font-weight: bold;
            font-size: 14px;
            padding: 8px;
        }}",
        COLORS.frost1.to_hex_string()
    ));
    day_label.style_context().add_provider(
        &label_css,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
    column.append(&day_label);

    let tasks_container = gtk::Box::new(gtk::Orientation::Vertical, 4);
    let container_css = gtk::CssProvider::new();
    container_css.load_from_data(&format!(
        "box {{
            background-color: {};
            border-radius: 8px;
            padding: 8px;
            min-height: 100px;
        }}",
        COLORS.polar2.to_hex_string()
    ));
    tasks_container.style_context().add_provider(
        &container_css,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
    column.append(&tasks_container);

    column
}
