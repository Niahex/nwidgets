use crate::services::mpris::{MprisService, MprisState, PlaybackStatus};
use crate::utils::icons;
use gtk::prelude::*;
use gtk4 as gtk;

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

        // Ajouter le contrôleur de scroll pour changer de piste
        let scroll_controller = gtk::EventControllerScroll::new(
            gtk::EventControllerScrollFlags::VERTICAL,
        );

        let container_clone = container.clone();
        scroll_controller.connect_scroll(move |_, _dx, dy| {
            // Vérifier que le widget est visible avant d'agir
            if !container_clone.is_visible() {
                return gtk::glib::Propagation::Proceed;
            }

            if dy < 0.0 {
                // Scroll vers le haut = piste suivante
                MprisService::next();
            } else if dy > 0.0 {
                // Scroll vers le bas = piste précédente
                MprisService::previous();
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
