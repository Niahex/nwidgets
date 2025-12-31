use crate::services::hyprland::{ActiveWindowChanged, HyprlandService};
use crate::utils::Icon;
use gpui::prelude::*;
use gpui::*;

pub struct ActiveWindowModule {
    hyprland: Entity<HyprlandService>,
}

impl ActiveWindowModule {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let hyprland = HyprlandService::global(cx);

        // Subscribe to active window changes
        cx.subscribe(
            &hyprland,
            |_this, _hyprland, _event: &ActiveWindowChanged, cx| {
                cx.notify();
            },
        )
        .detach();

        Self { hyprland }
    }

    /// Retourne le nom d'icône basé sur la classe de la fenêtre
    /// La classe est utilisée directement en minuscules comme nom d'icône
    /// Exemples: "Firefox" -> "firefox.svg", "discord" -> "discord.svg", "dev.zed.Zed" -> "dev.zed.zed.svg"
    fn get_icon_name(class: &str) -> String {
        let icon_name = class.to_lowercase();

        // Log seulement si l'icône n'existe pas
        let icon_path = format!("assets/{icon_name}.svg");
        if !std::path::Path::new(&icon_path).exists() {
            eprintln!(
                "[ActiveWindow] Icon not found for class '{class}' -> '{icon_path}'"
            );
        }

        icon_name
    }

    /// Extrait le titre court de la fenêtre (avant le premier " - ")
    fn extract_short_title(title: &str, max_chars: usize) -> String {
        let short_title = title
            .split(" - ")
            .next()
            .unwrap_or(title)
            .trim()
            .to_string();

        if short_title.chars().count() > max_chars {
            let truncated: String = short_title.chars().take(max_chars - 3).collect();
            format!("{truncated}...")
        } else {
            short_title
        }
    }

    /// Retourne la classe complète (pas de formatage)
    fn format_class_name(class: &str) -> String {
        class.to_string()
    }
}

impl Render for ActiveWindowModule {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let active_window = self.hyprland.read(cx).active_window();

        let theme = cx.global::<crate::theme::Theme>();

        let (icon_name, class_text, title_text) = if let Some(window) = active_window {
            let icon = Self::get_icon_name(&window.class);
            let class = Self::format_class_name(&window.class);
            let title = Self::extract_short_title(&window.title, 30);

            (icon, class, title)
        } else {
            // Pas de fenêtre active - afficher un placeholder
            ("nixos".to_string(), "NixOS".to_string(), "Nia".to_string())
        };

        div()
            .id("active-window-module")
            .flex()
            .gap_2()
            .items_center()
            .px_3()
            .py_2()
            .min_w(px(350.))
            .max_w(px(450.))
            .child(div().size(px(32.)).flex_shrink_0().child(
                Icon::new(icon_name).size(px(32.)).preserve_colors(true), // Préserver les couleurs des logos d'applications
            ))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_0p5()
                    .flex_1()
                    .min_w_0() // Pour permettre l'ellipsis
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(rgba(0xd8dee966)) // $snow1 à 40%
                            .overflow_hidden()
                            .whitespace_nowrap()
                            .child(class_text),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme.text)
                            .overflow_hidden()
                            .whitespace_nowrap()
                            .child(title_text),
                    ),
            )
    }
}
