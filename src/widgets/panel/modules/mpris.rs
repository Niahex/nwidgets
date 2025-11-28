use crate::services::mpris::{MprisService, MprisState, PlaybackStatus};
use crate::utils::icons;
use gtk::prelude::*;
use gtk4 as gtk;
use std::cell::Cell;
use std::rc::Rc;
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct MprisModule {
    pub container: gtk::Box,
    icon: gtk::Image,
    title_label: gtk::Label,
    artist_label: gtk::Label,
}

impl MprisModule {
    pub fn new() -> Self {
        let container = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        container.add_css_class("mpris-widget");

        // Icône de lecture
        let icon = icons::create_icon_with_size("play", Some(20));
        icon.add_css_class("mpris-icon");
        container.append(&icon);

        // Container pour le texte
        let text_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
        text_box.set_valign(gtk::Align::Center);

        let title_label = gtk::Label::new(None);
        title_label.add_css_class("mpris-title");
        title_label.set_halign(gtk::Align::Start);
        title_label.set_ellipsize(gtk::pango::EllipsizeMode::End);
        title_label.set_max_width_chars(30);

        let artist_label = gtk::Label::new(None);
        artist_label.add_css_class("mpris-artist");
        artist_label.set_halign(gtk::Align::Start);
        artist_label.set_ellipsize(gtk::pango::EllipsizeMode::End);
        artist_label.set_max_width_chars(30);

        text_box.append(&title_label);
        text_box.append(&artist_label);
        container.append(&text_box);

        // Masquer par défaut
        container.set_visible(false);

        // Ajouter le contrôleur de clic pour play/pause
        let gesture_click = gtk::GestureClick::new();
        gesture_click.set_button(gtk::gdk::ffi::GDK_BUTTON_PRIMARY as u32);

        gesture_click.connect_released(move |_, _, _, _| {
            MprisService::play_pause();
        });

        container.add_controller(gesture_click);

        // Ajouter le contrôleur de scroll pour volume/pistes
        // BOTH_AXES pour capturer vertical (volume) et horizontal (pistes)
        let scroll_controller = gtk::EventControllerScroll::new(
            gtk::EventControllerScrollFlags::BOTH_AXES,
        );

        // Debounce pour éviter les changements de piste trop rapides
        // On met Instant::now() - 1 seconde pour autoriser le premier événement
        let last_track_change = Rc::new(Cell::new(Instant::now() - Duration::from_secs(1)));
        let track_change_cooldown = Duration::from_millis(300); // 300ms entre chaque changement

        let container_clone = container.clone();
        let last_track_change_clone = Rc::clone(&last_track_change);
        scroll_controller.connect_scroll(move |_controller, dx, dy| {
            // Vérifier que le widget est visible avant d'agir
            if !container_clone.is_visible() {
                return gtk::glib::Propagation::Proceed;
            }

            let now = Instant::now();

            // Scroll horizontal (tilt gauche/droite) = Changer de piste
            if dx.abs() > 0.0 {
                // Vérifier le cooldown pour les changements de piste
                if now.duration_since(last_track_change_clone.get()) >= track_change_cooldown {
                    if dx < 0.0 {
                        println!("[MPRIS] Scroll LEFT -> Previous track");
                        MprisService::previous();
                    } else {
                        println!("[MPRIS] Scroll RIGHT -> Next track");
                        MprisService::next();
                    }
                    last_track_change_clone.set(now);
                } else {
                    println!("[MPRIS] Track change ignored (cooldown)");
                }
            }
            // Scroll vertical (haut/bas) = Ajuster le volume
            else if dy.abs() > 0.0 {
                if dy < 0.0 {
                    println!("[MPRIS] Scroll UP -> Volume up");
                    MprisService::volume_up();
                } else {
                    println!("[MPRIS] Scroll DOWN -> Volume down");
                    MprisService::volume_down();
                }
            }

            gtk::glib::Propagation::Stop
        });

        container.add_controller(scroll_controller);

        Self {
            container,
            icon,
            title_label,
            artist_label,
        }
    }

    pub fn update(&self, state: MprisState) {
        // Si rien ne joue, masquer le widget
        if state.status == PlaybackStatus::Stopped || state.metadata.title.is_empty() {
            self.container.set_visible(false);
            return;
        }

        // Afficher le widget
        self.container.set_visible(true);

        // Mettre à jour l'icône selon le statut
        let icon_name = match state.status {
            PlaybackStatus::Playing => "play",
            PlaybackStatus::Paused => "pause",
            PlaybackStatus::Stopped => "play",
        };

        if let Some(paintable) = icons::get_paintable_with_size(icon_name, Some(20)) {
            self.icon.set_paintable(Some(&paintable));
        }

        // Mettre à jour le titre
        self.title_label.set_text(&state.metadata.title);

        // Mettre à jour l'artiste
        if !state.metadata.artist.is_empty() {
            self.artist_label.set_text(&state.metadata.artist);
            self.artist_label.set_visible(true);
        } else {
            self.artist_label.set_visible(false);
        }
    }
}
