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
    // Extraire la valeur en pixels de manière sûre
    let s: f32 = config.size.into();
    let center = point(px(s / 2.0), px(s / 2.0));
    let radius = s / 2.0;

    // Créer le chemin du cercle qui va "mordre" le carré
    // On dessine un cercle complet, mais seul le quadrant dans les bounds sera visible
    let mut path_builder = PathBuilder::fill();

    // Dessiner un cercle en utilisant 4 arcs de 90° chacun
    // Point de départ: droite du cercle (3h)
    let start = point(center.x + px(radius), center.y);
    path_builder.move_to(start);

    // Arc de 90° vers le haut (12h)
    path_builder.arc_to(
        point(px(radius), px(radius)),
        px(0.0),
        false, // small arc
        false, // counter-clockwise
        point(center.x, center.y - px(radius)),
    );

    // Arc de 90° vers la gauche (9h)
    path_builder.arc_to(
        point(px(radius), px(radius)),
        px(0.0),
        false,
        false,
        point(center.x - px(radius), center.y),
    );

    // Arc de 90° vers le bas (6h)
    path_builder.arc_to(
        point(px(radius), px(radius)),
        px(0.0),
        false,
        false,
        point(center.x, center.y + px(radius)),
    );

    // Arc de 90° vers la droite (3h) - retour au point de départ
    path_builder.arc_to(point(px(radius), px(radius)), px(0.0), false, false, start);

    path_builder.close();

    // Appliquer une translation pour positionner le cercle selon le quadrant
    let translate_to = match config.position {
        CoveCornerPosition::TopLeft => {
            // Q1: Le centre du cercle doit être au coin inférieur droit des bounds
            point(bounds.origin.x + px(s / 2.0), bounds.origin.y + px(s / 2.0))
        }
        CoveCornerPosition::TopRight => {
            // Q2: Le centre du cercle doit être au coin inférieur gauche des bounds
            point(bounds.origin.x + px(s / 2.0), bounds.origin.y + px(s / 2.0))
        }
        CoveCornerPosition::BottomLeft => {
            // Q3: Le centre du cercle doit être au coin supérieur droit des bounds
            point(bounds.origin.x + px(s / 2.0), bounds.origin.y + px(s / 2.0))
        }
        CoveCornerPosition::BottomRight => {
            // Q4: Le centre du cercle doit être au coin supérieur gauche des bounds
            point(bounds.origin.x + px(s / 2.0), bounds.origin.y + px(s / 2.0))
        }
    };

    path_builder.translate(translate_to);

    // Construire et dessiner le chemin
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

    // Utiliser un content mask pour clipper au bounds exact
    window.with_content_mask(Some(ContentMask { bounds }), |window| {
        paint_cove_corner(window, cx, bounds, config);
    });
}
