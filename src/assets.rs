use anyhow::Result;
use gpui::*;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::theme::ActiveTheme;

/// Asset source pour GPUI
pub struct Assets {
    base: PathBuf,
    cache: RwLock<HashMap<String, &'static [u8]>>,
}

impl Assets {
    pub fn new(base: PathBuf) -> Self {
        Self {
            base,
            cache: RwLock::new(HashMap::new()),
        }
    }
}

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<std::borrow::Cow<'static, [u8]>>> {
        {
            let cache = self.cache.read();
            if let Some(data) = cache.get(path) {
                return Ok(Some(std::borrow::Cow::Borrowed(data)));
            }
        }

        match std::fs::read(self.base.join(path)) {
            Ok(data) => {
                let leaked_data: &'static [u8] = Box::leak(data.into_boxed_slice());
                let mut cache = self.cache.write();
                cache.insert(path.to_string(), leaked_data);
                Ok(Some(std::borrow::Cow::Borrowed(leaked_data)))
            }
            Err(_) => {
                // Fallback to none.svg if icon not found
                if path.starts_with("icons/") && path.ends_with(".svg") {
                    match std::fs::read(self.base.join("icons/none.svg")) {
                        Ok(data) => {
                            let leaked_data: &'static [u8] = Box::leak(data.into_boxed_slice());
                            Ok(Some(std::borrow::Cow::Borrowed(leaked_data)))
                        }
                        Err(e) => Err(e.into()),
                    }
                } else {
                    Err(anyhow::anyhow!("Asset not found: {}", path))
                }
            }
        }
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        std::fs::read_dir(self.base.join(path))
            .map(|entries| {
                entries
                    .filter_map(|entry| {
                        entry
                            .ok()
                            .and_then(|entry| entry.file_name().into_string().ok())
                            .map(SharedString::from)
                    })
                    .collect()
            })
            .map_err(|err| err.into())
    }
}

pub fn determine_assets_path() -> PathBuf {
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        PathBuf::from(manifest_dir)
    } else {
        std::env::current_exe()
            .ok()
            .and_then(|path| path.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."))
    }
}

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
#[derive(IntoElement)]
pub struct Icon {
    name: String,
    size: Pixels,
    color: Option<Hsla>,
    preserve_colors: bool,
}

impl Icon {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            size: px(16.),
            color: None,
            preserve_colors: false,
        }
    }

    pub fn size(mut self, size: Pixels) -> Self {
        self.size = size;
        self
    }

    pub fn color(mut self, color: impl Into<Hsla>) -> Self {
        self.color = Some(color.into());
        self
    }

    pub fn preserve_colors(mut self, preserve: bool) -> Self {
        self.preserve_colors = preserve;
        self
    }

    fn get_path(&self) -> Arc<str> {
        {
            let cache = ICON_CACHE.read();
            if let Some(path) = cache.get(&self.name) {
                return path.clone();
            }
        }

        let path = format!("{}/{}.svg", assets_dir().display(), self.name);

        if !std::path::Path::new(&path).exists() {
            log::warn!("Icon file not found: '{path}'");
        }

        let path_arc: Arc<str> = path.into();

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

        if self.preserve_colors {
            let path: Arc<Path> = Arc::from(PathBuf::from(path_str.as_ref()));
            img(path).size(self.size).flex_none().into_any_element()
        } else {
            let mut svg_element = svg().path(path_str).size(self.size);

            if let Some(color) = self.color {
                svg_element = svg_element.text_color(color);
            } else {
                svg_element = svg_element.text_color(cx.theme().white);
            }

            svg_element.into_any_element()
        }
    }
}
