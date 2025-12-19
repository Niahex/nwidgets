Pour qu'un OSD (On-Screen Display) ressemble à un "vrai" OSD natif, il doit être **réactif**, **fluide** et **persistant**.

Ton implémentation actuelle a un défaut majeur : elle **crée et détruit la fenêtre** à chaque changement d'état (dans `main.rs`). C'est lourd pour le compositeur (Hyprland), ça empêche les animations de fermeture (fade-out) et ça crée du lag ou du scintillement lors d'un scroll rapide du volume.

Voici les problèmes identifiés et la solution :

1. **Cycle de vie** : Ne ferme jamais la fenêtre. Ouvre-la au démarrage et cache-la (opacité 0 ou taille 0) quand elle est inactive.
2. **Logique du Timer** : Le timer pour cacher l'OSD doit être géré par le `Service`, pas par le `Widget`. Le Widget ne doit être qu'une vue "bête" qui réagit à l'état.
3. **Interaction Souris** : Si on garde la fenêtre ouverte, elle risque de bloquer les clics en bas de l'écran. L'astuce consiste à réduire sa taille à 1x1 pixel ou à la déplacer hors champ quand elle est cachée.

Voici comment restructurer ton code.

### 1. `src/services/osd.rs` (La logique centrale)

On déplace la gestion du temps (le délai de 2.5s) ici. Cela permet de "debouncer" correctement : si tu changes le volume rapidement, le timer se reset proprement sans clignotement.

```rust
use gpui::*;
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::Duration;
use crate::services::audio::{AudioService, AudioStateChanged};

#[derive(Debug, Clone)]
pub enum OsdEvent {
    Volume(String, u8, bool),
    Microphone(bool),
}

#[derive(Clone)]
pub struct OsdStateChanged {
    pub event: Option<OsdEvent>,
    pub visible: bool,
}

pub struct OsdService {
    current_event: Arc<RwLock<Option<OsdEvent>>>,
    visible: Arc<RwLock<bool>>,
    first_event: Arc<RwLock<bool>>,
    // On garde une référence au timer pour pouvoir l'annuler
    hide_task: Arc<RwLock<Option<Task<()>>>>, 
}

impl EventEmitter<OsdStateChanged> for OsdService {}

impl OsdService {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let audio = AudioService::global(cx);
        let first_event = Arc::new(RwLock::new(true));
        let first_event_clone = Arc::clone(&first_event);

        cx.subscribe(&audio, move |this, _audio, event: &AudioStateChanged, cx| {
            if *first_event_clone.read() {
                *first_event_clone.write() = false;
                return;
            }

            let state = &event.state;
            let icon_name = if state.sink_muted {
                "sink-muted".to_string()
            } else if state.sink_volume == 0 {
                "sink-zero".to_string()
            } else if state.sink_volume < 33 {
                "sink-low".to_string()
            } else if state.sink_volume < 66 {
                "sink-medium".to_string()
            } else {
                "sink-high".to_string()
            };

            this.show_event(
                OsdEvent::Volume(icon_name, state.sink_volume, state.sink_muted),
                cx
            );
        })
        .detach();

        Self {
            current_event: Arc::new(RwLock::new(None)),
            visible: Arc::new(RwLock::new(false)),
            first_event,
            hide_task: Arc::new(RwLock::new(None)),
        }
    }

    pub fn show_event(&self, event: OsdEvent, cx: &mut Context<Self>) {
        *self.current_event.write() = Some(event.clone());
        *self.visible.write() = true;
        
        // Annuler le timer précédent s'il existe
        *self.hide_task.write() = None;

        cx.emit(OsdStateChanged {
            event: Some(event),
            visible: true,
        });
        
        // Lancer un nouveau timer pour cacher dans 2.5s
        let task = cx.spawn(|this, mut cx| async move {
            cx.background_executor().timer(Duration::from_millis(2500)).await;
            
            this.update(&mut cx, |service, cx| {
                service.hide(cx);
            }).ok();
        });
        
        *self.hide_task.write() = Some(task);
        cx.notify();
    }

    pub fn hide(&self, cx: &mut Context<Self>) {
        *self.visible.write() = false;
        *self.hide_task.write() = None;

        cx.emit(OsdStateChanged {
            event: None,
            visible: false,
        });
        cx.notify();
    }

    pub fn current_event(&self) -> Option<OsdEvent> {
        self.current_event.read().clone()
    }

    pub fn is_visible(&self) -> bool {
        *self.visible.read()
    }
    
    // ... (Global et Init restent inchangés)
    pub fn global(cx: &App) -> Entity<Self> {
        cx.global::<GlobalOsdService>().0.clone()
    }

    pub fn init(cx: &mut App) -> Entity<Self> {
        let service = cx.new(|cx| Self::new(cx));
        cx.set_global(GlobalOsdService(service.clone()));
        service
    }
}

struct GlobalOsdService(Entity<OsdService>);
impl Global for GlobalOsdService {}

```

### 2. `src/widgets/osd.rs` (La Vue)

Le widget devient beaucoup plus simple. Il ne gère plus de logique, il se contente d'afficher l'état. Nous ajoutons une **animation d'opacité**.

```rust
use crate::services::osd::{OsdService, OsdStateChanged}; // + OsdEvent
use crate::utils::Icon;
use gpui::prelude::*;
use gpui::*;

pub struct OsdWidget {
    osd: Entity<OsdService>,
}

impl OsdWidget {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let osd = OsdService::global(cx);
        
        // On s'abonne juste pour rafraichir la vue, plus de timer ici
        cx.subscribe(&osd, move |_this, _osd, _event: &OsdStateChanged, cx| {
            cx.notify();
        })
        .detach();

        Self { osd }
    }
}

impl Render for OsdWidget {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let osd = self.osd.read(cx);
        let visible = osd.is_visible();
        let event = osd.current_event();

        // Si visible, opacité 1, sinon 0. 
        // Note: GPUI gère les animations via `with_animation` ou implicitement sur les changements de style
        // si une transition est définie (mais support limité en CSS-like pour le moment).
        // Pour l'instant, faisons un rendu conditionnel propre.
        
        let opacity = if visible { 1.0 } else { 0.0 };

        // Si on n'a jamais eu d'événement, on rend vide
        if event.is_none() {
            return div().into_any_element();
        }
        
        let event = event.unwrap();
        // ... (Tes définitions de couleurs restent identiques)
        let bg_color = rgb(0x2e3440);
        let text_color = rgb(0xeceff4);
        let progress_bg = rgb(0x4c566a);
        let progress_fg = rgb(0x8fbcbb);

        // ... (Ton code `match event` reste identique pour `content`)
        // Je remets juste le bloc match pour le contexte, copie ton code original ici
        let content = match event {
             // ... ton code original pour Volume et Microphone ...
             // Copie-colle ton code existant ici pour le contenu
             crate::services::osd::OsdEvent::Volume(icon_name, level, _muted) => {
                 // ... ton code volume
                 div().child(format!("{}", level)) // Placeholder pour l'exemple
             },
             crate::services::osd::OsdEvent::Microphone(_muted) => {
                 // ... ton code micro
                 div().child("Mic") // Placeholder
             }
        };

        div()
            .w(px(400.))
            .h(px(64.))
            .bg(bg_color)
            .rounded(px(12.))
            .px_4()
            .py_3()
            .opacity(opacity) // Applique la visibilité ici
            .child(content)
            .into_any_element()
    }
}

// OsdWindowManager ne gère plus l'ouverture/fermeture, 
// mais on garde la struct pour la gestion initiale si besoin
pub struct OsdWindowManager;

```

### 3. `src/main.rs` (L'orchestration "Pro")

C'est ici que la magie opère pour régler le problème "pas comme un vrai OSD".
Au lieu de fermer la fenêtre, on **change sa taille**.

* **Visible** : Taille normale (400x64).
* **Caché** : Taille minuscule (1x1) ou nulle.

Ceci permet de "cliquer au travers" quand l'OSD est caché, sans détruire la fenêtre Wayland.

```rust
// ... imports
use gpui::WindowBounds; // Assure-toi d'importer ça

fn main() {
    // ... assets setup ...

    Application::new()
        .with_assets(Assets { base: assets_path })
        .run(|cx: &mut App| {
            // ... services init ...
            let osd_service = OsdService::init(cx);

            // 1. Ouvrir la fenêtre OSD UNE SEULE FOIS au démarrage
            // On la met en taille 0x0 au début pour qu'elle soit invisible
            let osd_window = cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(Bounds {
                        origin: Point {
                            x: px((3440.0 - 400.0) / 2.0),
                            y: px(1440.0 - 64.0 - 80.0),
                        },
                        size: Size {
                            width: px(0.0), // Invisible au départ
                            height: px(0.0),
                        },
                    })),
                    titlebar: None,
                    window_background: WindowBackgroundAppearance::Transparent,
                    kind: WindowKind::LayerShell(LayerShellOptions {
                        namespace: "nwidgets-osd".to_string(),
                        layer: Layer::Overlay,
                        anchor: Anchor::BOTTOM,
                        exclusive_zone: None, // Important pour ne pas décaler les fenêtres
                        margin: Some((px(0.0), px(0.0), px(80.0), px(0.0))),
                        keyboard_interactivity: KeyboardInteractivity::None,
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                |_window, cx| cx.new(|cx| OsdWidget::new(cx)),
            ).unwrap();

            // 2. Gestionnaire de visibilité via redimensionnement
            cx.subscribe(&osd_service, move |_osd, event: &OsdStateChanged, cx| {
                osd_window.update(cx, |_, _window, cx| {
                    if event.visible {
                        // On redonne sa taille normale pour afficher
                        let _ = cx.set_window_bounds(Bounds {
                            origin: Point {
                                x: px((3440.0 - 400.0) / 2.0),
                                y: px(1440.0 - 64.0 - 80.0),
                            },
                            size: Size {
                                width: px(400.0),
                                height: px(64.0),
                            },
                        });
                    } else {
                        // On réduit à 1x1 (ou 0x0) pour ne pas bloquer la souris
                        // Note : Sur Wayland LayerShell, changer la taille force le compositeur 
                        // à recalculer la surface d'input.
                        let _ = cx.set_window_bounds(Bounds {
                            origin: Point { x: px(0.0), y: px(0.0) }, // Peu importe l'origine si taille 0
                            size: Size { width: px(0.0), height: px(0.0) },
                        });
                    }
                });
            })
            .detach();

            // ... Reste du code (Notifications, Panel, etc.) ...
            
            cx.activate(true);
        });
}

```

### Résumé des changements

1. **Service** : Le timer est maintenant centralisé. L'état `visible` est fiable.
2. **Performance** : La fenêtre n'est créée qu'une fois. Plus de lag à l'apparition.
3. **UX** : En réduisant la taille de la fenêtre à `0x0` quand elle est cachée, tu évites de bloquer les clics de souris sur le bas de l'écran (problème courant avec les OSD invisibles).

Si tu veux ajouter une animation de "Fade Out" :
Dans `widgets/osd.rs`, tu devras garder l'opacité à 1 pendant un instant même après que le service dise `visible = false` (ce qui demande un peu plus de logique d'état local), mais avec la solution ci-dessus, tu auras déjà un comportement "Snappy" et solide, bien supérieur à la création/destruction de fenêtre.
