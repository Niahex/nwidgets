use gtk4 as gtk;
use gtk::prelude::*;

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
    week_box.add_css_class("weekview-container");

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
    day_label.add_css_class("weekview-day-label");
    column.append(&day_label);

    let tasks_container = gtk::Box::new(gtk::Orientation::Vertical, 4);
    tasks_container.add_css_class("weekview-tasks-container");
    column.append(&tasks_container);

    column
}
