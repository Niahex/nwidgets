use gtk::prelude::*;
use gtk4 as gtk;
use chrono::Local;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use super::views::ViewMode;
use crate::services::{Task, TasksService};

pub fn show_add_task_form(
    view_container: &gtk::Box,
    carousel_container: &gtk::Box,
    current_view: &Rc<RefCell<ViewMode>>,
) {
    // Clear current content
    while let Some(child) = view_container.first_child() {
        view_container.remove(&child);
    }

    // Hide carousel while in form view
    carousel_container.set_visible(false);

    // Create form container
    let form_container = gtk::Box::new(gtk::Orientation::Vertical, 16);
    form_container.add_css_class("task-form-container");
    form_container.set_margin_start(24);
    form_container.set_margin_end(24);
    form_container.set_margin_top(24);
    form_container.set_margin_bottom(24);
    form_container.set_vexpand(true);

    // Scrolled window for the form
    let scrolled = gtk::ScrolledWindow::new();
    scrolled.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    scrolled.set_vexpand(true);
    scrolled.set_child(Some(&form_container));

    // Form title
    let form_title = gtk::Label::new(Some("New Task"));
    form_title.add_css_class("task-form-title");
    form_title.set_halign(gtk::Align::Start);
    form_container.append(&form_title);

    // Task name
    let name_label = gtk::Label::new(Some("Task Name"));
    name_label.add_css_class("task-form-label");
    name_label.set_halign(gtk::Align::Start);
    form_container.append(&name_label);

    let name_entry = gtk::Entry::new();
    name_entry.add_css_class("task-form-entry");
    name_entry.set_placeholder_text(Some("Enter task name..."));
    form_container.append(&name_entry);

    // Date selection
    let date_label = gtk::Label::new(Some("Due Date"));
    date_label.add_css_class("task-form-label");
    date_label.set_halign(gtk::Align::Start);
    form_container.append(&date_label);

    let date_button = gtk::Button::new();
    date_button.add_css_class("task-form-date-button");
    let today = Local::now().format("%Y-%m-%d").to_string();
    date_button.set_label(&today);

    // Create calendar popover
    let calendar_popover = gtk::Popover::new();
    let calendar = gtk::Calendar::new();
    calendar.add_css_class("task-form-calendar");
    calendar_popover.set_child(Some(&calendar));
    calendar_popover.set_parent(&date_button);

    // Update date button when calendar date is selected
    let date_button_clone = date_button.clone();
    let calendar_popover_clone = calendar_popover.clone();
    calendar.connect_day_selected(move |cal| {
        let date = cal.date();
        let formatted_date = format!("{:04}-{:02}-{:02}",
            date.year(),
            date.month() as u32,
            date.day_of_month()
        );
        date_button_clone.set_label(&formatted_date);
        calendar_popover_clone.popdown();
    });

    // Show calendar when button is clicked
    date_button.connect_clicked(move |_| {
        calendar_popover.popup();
    });

    form_container.append(&date_button);

    // Priority selection
    let priority_label = gtk::Label::new(Some("Priority"));
    priority_label.add_css_class("task-form-label");
    priority_label.set_halign(gtk::Align::Start);
    form_container.append(&priority_label);

    let priority_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let priorities = vec![
        ("Low", "priority-low"),
        ("Medium", "priority-medium"),
        ("High", "priority-high"),
    ];

    let priority_selected = Rc::new(Cell::new("Medium"));
    let mut first_button: Option<gtk::ToggleButton> = None;

    for (priority, css_class) in priorities {
        let btn = if let Some(ref first) = first_button {
            // Create button grouped with the first one (radio button behavior)
            gtk::ToggleButton::builder()
                .label(priority)
                .group(first)
                .build()
        } else {
            // First button
            gtk::ToggleButton::new()
        };

        btn.set_label(priority);
        btn.add_css_class("task-form-priority-btn");
        btn.add_css_class(css_class);

        if priority == "Medium" {
            btn.set_active(true);
        }

        let priority_selected_clone = Rc::clone(&priority_selected);
        btn.connect_toggled(move |toggle_btn| {
            if toggle_btn.is_active() {
                priority_selected_clone.set(priority);
            }
        });

        // Store first button for grouping
        if first_button.is_none() {
            first_button = Some(btn.clone());
        }

        priority_box.append(&btn);
    }
    form_container.append(&priority_box);

    // Project selection
    let project_label = gtk::Label::new(Some("Project"));
    project_label.add_css_class("task-form-label");
    project_label.set_halign(gtk::Align::Start);
    form_container.append(&project_label);

    let project_entry = gtk::Entry::new();
    project_entry.add_css_class("task-form-entry");
    project_entry.set_placeholder_text(Some("Enter project name..."));
    form_container.append(&project_entry);

    // Category selection
    let category_label = gtk::Label::new(Some("Category"));
    category_label.add_css_class("task-form-label");
    category_label.set_halign(gtk::Align::Start);
    form_container.append(&category_label);

    let category_entry = gtk::Entry::new();
    category_entry.add_css_class("task-form-entry");
    category_entry.set_placeholder_text(Some("Enter category..."));
    form_container.append(&category_entry);

    // Description
    let description_label = gtk::Label::new(Some("Description (Optional)"));
    description_label.add_css_class("task-form-label");
    description_label.set_halign(gtk::Align::Start);
    form_container.append(&description_label);

    let description_frame = gtk::Frame::new(None);
    description_frame.add_css_class("task-form-description-frame");
    let description_view = gtk::TextView::new();
    description_view.add_css_class("task-form-description");
    description_view.set_wrap_mode(gtk::WrapMode::Word);
    description_view.set_pixels_above_lines(4);
    description_view.set_pixels_below_lines(4);
    description_view.set_left_margin(8);
    description_view.set_right_margin(8);
    description_view.set_height_request(100);
    description_frame.set_child(Some(&description_view));
    form_container.append(&description_frame);

    // Buttons
    let button_box = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    button_box.set_halign(gtk::Align::End);
    button_box.set_margin_top(16);

    let cancel_btn = gtk::Button::with_label("Cancel");
    cancel_btn.add_css_class("task-form-cancel-btn");

    let save_btn = gtk::Button::with_label("Save Task");
    save_btn.add_css_class("task-form-save-btn");

    button_box.append(&cancel_btn);
    button_box.append(&save_btn);
    form_container.append(&button_box);

    // Add scrolled window to view container
    view_container.append(&scrolled);

    // Cancel button - restore original view
    let view_container_clone = view_container.clone();
    let carousel_container_clone = carousel_container.clone();
    let current_view_clone = Rc::clone(current_view);
    cancel_btn.connect_clicked(move |_| {
        // Clear form
        while let Some(child) = view_container_clone.first_child() {
            view_container_clone.remove(&child);
        }

        // Restore view
        carousel_container_clone.set_visible(true);
        super::update_view(&view_container_clone, *current_view_clone.borrow());
    });

    // Save button - save task and restore view
    let view_container_clone2 = view_container.clone();
    let carousel_container_clone2 = carousel_container.clone();
    let current_view_clone2 = Rc::clone(current_view);
    let name_entry_clone = name_entry.clone();
    let date_button_clone = date_button.clone();
    let project_entry_clone = project_entry.clone();
    let category_entry_clone = category_entry.clone();
    let description_view_clone = description_view.clone();
    let priority_selected_clone = Rc::clone(&priority_selected);
    save_btn.connect_clicked(move |_| {
        let name = name_entry_clone.text().to_string();
        let due_date = date_button_clone.label().unwrap().to_string();
        let priority = priority_selected_clone.get().to_string();
        let project = project_entry_clone.text().to_string();
        let category = category_entry_clone.text().to_string();

        let buffer = description_view_clone.buffer();
        let (start, end) = buffer.bounds();
        let description = buffer.text(&start, &end, false).to_string();

        // Create and save the task
        let task = Task::new(
            name.clone(),
            description,
            due_date.clone(),
            priority.clone(),
            project,
            category,
        );

        match TasksService::add_task(task) {
            Ok(_) => {
                println!("[TASKER] Task saved!");
                println!("  Name: {}", name);
                println!("  Date: {}", due_date);
                println!("  Priority: {}", priority);
            }
            Err(e) => {
                eprintln!("[TASKER] Error saving task: {}", e);
            }
        }

        // Clear form
        while let Some(child) = view_container_clone2.first_child() {
            view_container_clone2.remove(&child);
        }

        // Restore view
        carousel_container_clone2.set_visible(true);
        super::update_view(&view_container_clone2, *current_view_clone2.borrow());
    });

    // Focus on name entry
    name_entry.grab_focus();
}
