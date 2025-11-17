use gpui::{point, px, App, Bounds, Hsla, PathBuilder, Pixels, Window};

/// Position du coin arrondi inversé (cove/concave)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoveCornerPosition {
    /// Coin supérieur gauche (Q1: x≤S/2 et y≥S/2)
    TopLeft,
    /// Coin supérieur droit (Q2: x≥S/2 et y≥S/2)
    TopRight,
    /// Coin inférieur gauche (Q3: x≤S/2 et y≤S/2)
    BottomLeft,
    /// Coin inférieur droit (Q4: x≥S/2 et y≤S/2)
    BottomRight,
}

/// Configuration pour un coin arrondi inversé (cove)
#[derive(Clone, Debug)]
pub struct CoveCornerConfig {
    /// Taille du carré (S×S)
    pub size: Pixels,
    /// Couleur de remplissage du coin (la couleur qui "mord" le carré)
    pub fill_color: Hsla,
    /// Position du coin
    pub position: CoveCornerPosition,
}

impl CoveCornerConfig {
    /// Crée une nouvelle configuration de coin cove
    pub fn new(size: Pixels, fill_color: Hsla, position: CoveCornerPosition) -> Self {
        Self {
            size,
            fill_color,
            position,
        }
    }
}

/// Génère et dessine un coin arrondi inversé (cove/concave) selon la formule mathématique:
///
/// Coin = (Carré S×S ∖ Disque(centre=(S/2,S/2), rayon=S/2)) ∩ Quadrant
///
/// # Méthode mathématique:
/// 1. Soit C le carré plein S×S
/// 2. Soit D le disque plein de centre (S/2, S/2) et rayon R=S/2
/// 3. Forme intermédiaire F = C ∖ D (le carré avec un trou circulaire)
/// 4. Coin final = F ∩ Quadrant (intersection avec le quadrant souhaité)
///
/// # Implémentation avec PathBuilder:
/// On dessine uniquement la partie du cercle visible dans le quadrant souhaité,
/// créant ainsi l'effet de "découpe" circulaire (cove).
pub fn paint_cove_corner(
    window: &mut Window,
    _cx: &mut App,
    bounds: Bounds<Pixels>,
    config: &CoveCornerConfig,
) {
    // Extraire la valeur en pixels
    let s: f32 = config.size.into();
    let radius = s / 2.0; // Rayon de l'arc = moitié de la taille

    let mut path_builder = PathBuilder::fill();

    match config.position {
        CoveCornerPosition::TopRight => {
            // SVG: corner top right - forme en L avec arrondi concave en haut à droite
            // Point de départ : coin en haut à droite
            path_builder.move_to(point(bounds.origin.x + bounds.size.width, bounds.origin.y));

            // Ligne horizontale vers la gauche (toute la largeur)
            path_builder.line_to(point(bounds.origin.x, bounds.origin.y));

            // Ligne verticale vers le bas (jusqu'au milieu)
            path_builder.line_to(point(bounds.origin.x, bounds.origin.y + px(radius)));

            // Arc concave : rayon rx=radius, ry=radius, x-axis-rotation=0, large-arc=0, sweep=1
            path_builder.arc_to(
                point(px(radius), px(radius)),
                px(0.0),
                false, // large arc = 0
                true,  // sweep = 1 (sens horaire)
                point(bounds.origin.x + px(radius), bounds.origin.y + bounds.size.height),
            );

            // Ligne horizontale vers la droite (jusqu'au bout)
            path_builder.line_to(point(bounds.origin.x + bounds.size.width, bounds.origin.y + bounds.size.height));

            // Ligne verticale vers le haut (retour au point de départ)
            path_builder.line_to(point(bounds.origin.x + bounds.size.width, bounds.origin.y));

            path_builder.close();
        }
        CoveCornerPosition::TopLeft => {
            // SVG: corner top left - forme en L avec arrondi concave en haut à gauche
            // Point de départ : coin en haut à gauche
            path_builder.move_to(point(bounds.origin.x, bounds.origin.y));

            // Ligne verticale vers le bas (toute la hauteur)
            path_builder.line_to(point(bounds.origin.x, bounds.origin.y + bounds.size.height));

            // Ligne horizontale vers la droite (jusqu'au milieu)
            path_builder.line_to(point(bounds.origin.x + px(radius), bounds.origin.y + bounds.size.height));

            // Arc concave
            path_builder.arc_to(
                point(px(radius), px(radius)),
                px(0.0),
                false,
                true,
                point(bounds.origin.x + bounds.size.width, bounds.origin.y + px(radius)),
            );

            // Ligne verticale vers le haut (jusqu'au bout)
            path_builder.line_to(point(bounds.origin.x + bounds.size.width, bounds.origin.y));

            // Ligne horizontale vers la gauche (retour au point de départ)
            path_builder.line_to(point(bounds.origin.x, bounds.origin.y));

            path_builder.close();
        }
        CoveCornerPosition::BottomLeft | CoveCornerPosition::BottomRight => {
            // TODO: Implémenter si nécessaire
        }
    }

    if let Ok(path) = path_builder.build() {
        window.paint_path(path, config.fill_color);
    }
}

/// Dessine un coin cove avec clipping pour s'assurer que seul le quadrant souhaité est visible
pub fn paint_cove_corner_clipped(
    window: &mut Window,
    cx: &mut App,
    bounds: Bounds<Pixels>,
    config: &CoveCornerConfig,
) {
    use gpui::ContentMask;

    // DEBUG: Dessiner un fond rouge pour voir les bounds
    use gpui::{fill, rgb};
    window.paint_quad(fill(bounds, rgb(0xff0000)));

    // Utiliser un content mask pour clipper au bounds exact
    window.with_content_mask(Some(ContentMask { bounds }), |window| {
        paint_cove_corner(window, cx, bounds, config);
    });
}
