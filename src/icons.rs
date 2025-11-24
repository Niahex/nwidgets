use gtk4::prelude::*;
use gtk4::{gdk, IconTheme, Image};

pub fn setup_icon_theme() {
    if let Some(display) = gdk::Display::default() {
        let icon_theme = IconTheme::for_display(&display);

        // Ajouter les chemins d'icônes NixOS
        icon_theme.add_search_path("/run/current-system/sw/share/icons");
        icon_theme.add_search_path("/home/nia/.local/share/icons");

        // Définir le thème Nordzy
        icon_theme.set_theme_name(Some("Nordzy-turquoise-dark"));
    }
}

pub fn create_icon(icon_name: &str, size: i32) -> Image {
    let image = Image::new();

    // Debug: vérifier si l'icône existe
    if let Some(display) = gdk::Display::default() {
        let icon_theme = IconTheme::for_display(&display);
        let has_icon = icon_theme.has_icon(icon_name);
        println!(
            "[ICON DEBUG] Icon '{}' exists in theme: {}",
            icon_name, has_icon
        );

        // Afficher aussi le nom du thème actuel
        let theme_name = icon_theme.theme_name();
        println!("[ICON DEBUG] Current theme: {}", theme_name);

        // Essayer de lookup l'icône et l'utiliser directement comme paintable
        let icon_paintable = icon_theme.lookup_icon(
            icon_name,
            &[],
            size,
            1,
            gtk4::TextDirection::None,
            gtk4::IconLookupFlags::empty(),
        );

        let file = icon_paintable.file();
        if let Some(file) = file {
            if let Some(path) = file.path() {
                println!("[ICON DEBUG] Icon path found: {:?}", path);
            } else {
                println!("[ICON DEBUG] File exists but no path");
            }
        } else {
            println!("[ICON DEBUG] No file found for icon '{}', using paintable directly", icon_name);
        }

        // Utiliser directement le paintable au lieu de set_icon_name
        image.set_paintable(Some(&icon_paintable));
    }

    image.set_pixel_size(size);
    image
}

pub fn get_icon_paintable(icon_name: &str, size: i32) -> Option<gtk4::gdk::Paintable> {
    if let Some(display) = gdk::Display::default() {
        let icon_theme = IconTheme::for_display(&display);
        let icon_paintable = icon_theme.lookup_icon(
            icon_name,
            &[],
            size,
            1,
            gtk4::TextDirection::None,
            gtk4::IconLookupFlags::empty(),
        );
        Some(icon_paintable.upcast())
    } else {
        None
    }
}
