use crate::theme::ActiveTheme;
use gpui::*;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Cache global des icônes SVG chargées
static ICON_CACHE: Lazy<RwLock<HashMap<String, Arc<str>>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

/// Répertoire des assets (peut être overridé via variable d'environnement)
fn assets_dir() -> PathBuf {
    std::env::var("NWIDGETS_ASSETS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("assets"))
}

/// Composant Icon qui charge dynamiquement les SVG depuis le dossier assets/
///
/// # Utilisation
///
/// ```rust
/// Icon::new("spotify")          // Charge assets/spotify.svg
/// Icon::new("sink-high")        // Charge assets/sink-high.svg
///     .size(px(24.))
///     .color(rgb(0xeceff4))
/// ```
///
/// # Ajout d'icônes
///
/// Déposez simplement un fichier SVG dans `assets/` et utilisez-le :
///
/// ```bash
/// cp mon-icone.svg assets/discord-nitro.svg
/// ```
///
/// ```rust
/// Icon::new("discord-nitro")  // Fonctionne immédiatement !
/// ```
#[derive(IntoElement)]
pub struct Icon {
    name: String,
    size: Pixels,
    color: Option<Hsla>,
    /// Si true, préserve les couleurs originales du SVG (pour les logos multi-couleurs)
    preserve_colors: bool,
}

impl Icon {
    /// Crée une nouvelle icône depuis un nom de fichier (sans l'extension .svg)
    ///
    /// # Exemples
    ///
    /// ```rust
    /// Icon::new("firefox")
    /// Icon::new("sink-muted")
    /// Icon::new("bluetooth-active")
    /// ```
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            size: px(16.),
            color: None,
            preserve_colors: false,
        }
    }

    /// Définit la taille de l'icône en pixels
    pub fn size(mut self, size: Pixels) -> Self {
        self.size = size;
        self
    }

    /// Définit la couleur de l'icône
    /// Note: Ne pas utiliser avec preserve_colors(true)
    pub fn color(mut self, color: impl Into<Hsla>) -> Self {
        self.color = Some(color.into());
        self
    }

    /// Préserve les couleurs originales du SVG (pour les logos multi-couleurs)
    /// Utiliser pour les icônes d'applications qui ont leurs propres couleurs
    pub fn preserve_colors(mut self, preserve: bool) -> Self {
        self.preserve_colors = preserve;
        self
    }

    /// Récupère le chemin de l'icône (avec cache)
    fn get_path(&self) -> Arc<str> {
        // Check cache first
        {
            let cache = ICON_CACHE.read();
            if let Some(path) = cache.get(&self.name) {
                return path.clone();
            }
        }

        // Not in cache, build path and cache it
        let path = format!("{}/{}.svg", assets_dir().display(), self.name);

        // Check if file exists
        if !std::path::Path::new(&path).exists() {
            log::warn!("Icon file not found: '{path}'");
        }

        let path_arc: Arc<str> = path.into();

        // Store in cache
        {
            let mut cache = ICON_CACHE.write();
            cache.insert(self.name.clone(), path_arc.clone());
        }

        path_arc
    }
}

impl RenderOnce for Icon {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let path_str = self.get_path();

        // GPUI rend les SVG comme des alpha masks et applique une couleur unique
        // Pour les SVG multi-couleurs (logos d'apps), on doit utiliser img() au lieu de svg()
        // Comme Zed le fait (voir zed/crates/ui/src/components/icon.rs:121-123)
        if self.preserve_colors {
            // Utiliser img() pour préserver les couleurs originales du SVG
            let path: Arc<Path> = Arc::from(PathBuf::from(path_str.as_ref()));
            img(path).size(self.size).flex_none().into_any_element()
        } else {
            // Utiliser svg() pour les icônes monochromes avec recoloriage
            let mut svg_element = svg().path(path_str).size(self.size);

            if let Some(color) = self.color {
                svg_element = svg_element.text_color(color);
            } else {
                // Couleur par défaut si aucune n'est spécifiée
                svg_element = svg_element.text_color(cx.theme().white);
            }

            svg_element.into_any_element()
        }
    }
}
