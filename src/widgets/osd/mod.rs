use crate::services::osd::{OsdEvent, OsdEventService};
use crate::utils::icons;
use gtk::prelude::*;
use gtk4 as gtk;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use std::cell::RefCell;
use std::rc::Rc;

pub fn create_osd_window(application: &gtk::Application) -> gtk::ApplicationWindow {
    let window = gtk::ApplicationWindow::builder()
        .application(application)
        .build();

    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.set_anchor(Edge::Bottom, true);
    window.set_margin(Edge::Bottom, 80);
    window.set_keyboard_mode(KeyboardMode::None);

    let container = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    container.set_width_request(400);
    container.set_height_request(64);
    container.set_halign(gtk::Align::Center);
    container.set_valign(gtk::Align::Center);
    container.set_margin_start(16);
    container.set_margin_end(16);
    container.add_css_class("osd-container");

    window.set_child(Some(&container));
    window.set_visible(false);

    // Compteur de génération pour invalider les anciens timeouts
    let generation: Rc<RefCell<u32>> = Rc::new(RefCell::new(0));

    let show_osd = {
        let window = window.clone();
        let generation = Rc::clone(&generation);
        move |content: gtk::Widget| {
            // Remplacer le contenu
            let container: gtk::Box = window.child().unwrap().downcast().unwrap();
            while let Some(child) = container.first_child() {
                container.remove(&child);
            }
            container.append(&content);

            // Afficher
            window.set_visible(true);

            // Incrémenter génération pour invalider l'ancien timeout
            *generation.borrow_mut() += 1;
            let current_gen = *generation.borrow();

            // Nouveau timeout 2.5s
            let window_clone = window.clone();
            let generation_clone = Rc::clone(&generation);
            glib::timeout_add_seconds_local(2, move || {
                // Ne cacher que si c'est toujours la même génération
                if *generation_clone.borrow() == current_gen {
                    window_clone.set_visible(false);
                }
                glib::ControlFlow::Break
            });
        }
    };

    // Écouter les événements OSD
    let osd_rx = OsdEventService::init();
    let (async_tx, async_rx) = async_channel::unbounded();

    std::thread::spawn(move || {
        while let Ok(event) = osd_rx.recv() {
            if async_tx.send_blocking(event).is_err() {
                break;
            }
        }
    });

    glib::MainContext::default().spawn_local(async move {
        while let Ok(event) = async_rx.recv().await {
            let content = match event {
                OsdEvent::Volume(icon_name, level, _muted) => create_volume_osd(&icon_name, level),
                OsdEvent::Microphone(muted) => create_mic_osd(muted),
                OsdEvent::CapsLock(enabled) => create_capslock_osd(enabled),
                OsdEvent::NumLock(enabled) => create_numlock_osd(enabled),
                OsdEvent::Clipboard => create_clipboard_osd(),
                OsdEvent::DictationStarted => create_dictation_osd(true),
                OsdEvent::DictationStopped => create_dictation_osd(false),
                OsdEvent::SttRecording => create_stt_recording_osd(),
                OsdEvent::SttProcessing => create_stt_processing_osd(),
                OsdEvent::SttComplete(text) => create_stt_complete_osd(&text),
                OsdEvent::SttError(error) => create_stt_error_osd(&error),
            };
            show_osd(content);
        }
    });

    window
}

fn create_volume_osd(icon_name: &str, level: u8) -> gtk::Widget {
    let container = gtk::Box::new(gtk::Orientation::Horizontal, 12);

    let icon = icons::create_icon(icon_name);
    icon.add_css_class("osd-icon");
    container.append(&icon);

    // Progress bar
    let progress = gtk::ProgressBar::new();
    progress.set_fraction(level as f64 / 100.0);
    progress.set_hexpand(true);
    progress.add_css_class("osd-progress");
    container.append(&progress);

    // Percentage
    let percent_label = gtk::Label::new(Some(&format!("{level}%")));
    percent_label.add_css_class("osd-text");
    container.append(&percent_label);

    container.upcast()
}

fn create_mic_osd(muted: bool) -> gtk::Widget {
    let container = gtk::Box::new(gtk::Orientation::Horizontal, 12);

    let icon_name = if muted {
        "microphone-sensitivity-muted"
    } else {
        "audio-input-microphone"
    };

    let icon = icons::create_icon(icon_name);
    icon.add_css_class("osd-icon");
    container.append(&icon);

    let text = if muted { "MIC OFF" } else { "MIC ON" };
    let text_label = gtk::Label::new(Some(text));
    text_label.add_css_class("osd-text");
    container.append(&text_label);

    container.upcast()
}

fn create_dictation_osd(started: bool) -> gtk::Widget {
    let container = gtk::Box::new(gtk::Orientation::Horizontal, 12);

    let icon = icons::create_icon("audio-input-microphone");
    icon.add_css_class("osd-icon");
    container.append(&icon);

    let text = if started {
        "Transcription Started"
    } else {
        "Transcription Stopped"
    };
    let text_label = gtk::Label::new(Some(text));
    text_label.add_css_class("osd-text");
    container.append(&text_label);

    container.upcast()
}

fn create_capslock_osd(enabled: bool) -> gtk::Widget {
    let container = gtk::Box::new(gtk::Orientation::Horizontal, 12);

    let icon_name = if enabled {
        "capslock-on"
    } else {
        "capslock-off"
    };
    let icon = icons::create_icon(icon_name);
    icon.add_css_class("osd-icon");
    container.append(&icon);

    let text = if enabled {
        "CAPS LOCK ON"
    } else {
        "CAPS LOCK OFF"
    };
    let text_label = gtk::Label::new(Some(text));
    text_label.add_css_class("osd-text");
    container.append(&text_label);

    container.upcast()
}

fn create_numlock_osd(enabled: bool) -> gtk::Widget {
    let container = gtk::Box::new(gtk::Orientation::Horizontal, 12);

    let icon_label = gtk::Label::new(Some("󰎠")); // nerd font numpad icon
    icon_label.add_css_class("osd-icon");
    container.append(&icon_label);

    let text = if enabled {
        "NUM LOCK ON"
    } else {
        "NUM LOCK OFF"
    };
    let text_label = gtk::Label::new(Some(text));
    text_label.add_css_class("osd-text");
    container.append(&text_label);

    container.upcast()
}

fn create_clipboard_osd() -> gtk::Widget {
    let container = gtk::Box::new(gtk::Orientation::Horizontal, 12);

    let icon = icons::create_icon("copy");
    icon.add_css_class("osd-icon");
    container.append(&icon);

    let text_label = gtk::Label::new(Some("Copy"));
    text_label.add_css_class("osd-text");
    container.append(&text_label);

    container.upcast()
}

fn create_stt_recording_osd() -> gtk::Widget {
    let container = gtk::Box::new(gtk::Orientation::Horizontal, 12);

    let icon = icons::create_icon("source-recorder");
    icon.add_css_class("osd-icon");
    icon.add_css_class("recording-pulse");
    container.append(&icon);

    let text_label = gtk::Label::new(Some("Recording..."));
    text_label.add_css_class("osd-text");
    container.append(&text_label);

    container.upcast()
}

fn create_stt_processing_osd() -> gtk::Widget {
    let container = gtk::Box::new(gtk::Orientation::Horizontal, 12);

    let icon = icons::create_icon("source-processing");
    icon.add_css_class("osd-icon");
    container.append(&icon);

    let text_label = gtk::Label::new(Some("Processing..."));
    text_label.add_css_class("osd-text");
    container.append(&text_label);

    container.upcast()
}

fn create_stt_complete_osd(text: &str) -> gtk::Widget {
    let container = gtk::Box::new(gtk::Orientation::Horizontal, 12);

    let icon = icons::create_icon("clipboard");
    icon.add_css_class("osd-icon");
    container.append(&icon);

    let text_label = gtk::Label::new(Some(text));
    text_label.add_css_class("osd-text");
    container.append(&text_label);

    container.upcast()
}

fn create_stt_error_osd(error: &str) -> gtk::Widget {
    let container = gtk::Box::new(gtk::Orientation::Horizontal, 12);

    let icon = icons::create_icon("dialog-error");
    icon.add_css_class("osd-icon");
    container.append(&icon);

    let text_label = gtk::Label::new(Some(&format!("Error: {error}")));
    text_label.add_css_class("osd-text");
    container.append(&text_label);

    container.upcast()
}
