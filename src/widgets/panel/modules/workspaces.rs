use gtk4 as gtk;
use gtk::prelude::*;
use crate::services::hyprland::Workspace;

#[derive(Clone)]
pub struct WorkspacesModule {
    pub container: gtk::Box,
    workspace_buttons: Vec<gtk::Label>,
}

impl WorkspacesModule {
    pub fn new() -> Self {
        let container = gtk::Box::new(gtk::Orientation::Horizontal, 8); // gap-2 (8px)
        container.set_halign(gtk::Align::Center);
        container.add_css_class("workspaces-widget");

        // Créer 8 labels pour les workspaces (on affiche max 8)
        let workspace_buttons: Vec<gtk::Label> = (1..=8)
            .map(|i| {
                let label = gtk::Label::new(Some(&i.to_string()));
                label.set_width_request(32);  // w-8 (32px)
                label.set_height_request(32); // h-8 (32px)
                label.set_halign(gtk::Align::Center);
                label.set_valign(gtk::Align::Center);
                label.add_css_class("workspace-button");
                label.add_css_class("workspace-inactive");
                label.set_visible(false); // Cachés par défaut
                container.append(&label);
                label
            })
            .collect();

        Self {
            container,
            workspace_buttons,
        }
    }

    pub fn update(&self, workspaces: Vec<Workspace>, active_workspace: i32) {
        // Trier les workspaces : 1-6 d'abord, puis les autres
        let mut sorted_workspaces = workspaces.clone();
        sorted_workspaces.sort_by(|a, b| match (a.id <= 6, b.id <= 6) {
            (true, true) => a.id.cmp(&b.id),
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            (false, false) => a.id.cmp(&b.id),
        });

        // Cacher tous les boutons d'abord
        for button in &self.workspace_buttons {
            button.set_visible(false);
        }

        // Afficher seulement les workspaces existants (max 8)
        for (idx, ws) in sorted_workspaces.iter().take(8).enumerate() {
            if let Some(button) = self.workspace_buttons.get(idx) {
                button.set_text(&ws.id.to_string());
                button.set_visible(true);

                // Retirer toutes les classes de style
                button.remove_css_class("workspace-active");
                button.remove_css_class("workspace-inactive");

                // Ajouter la classe appropriée selon si c'est le workspace actif
                if ws.id == active_workspace {
                    button.add_css_class("workspace-active");
                } else {
                    button.add_css_class("workspace-inactive");
                }
            }
        }
    }
}
