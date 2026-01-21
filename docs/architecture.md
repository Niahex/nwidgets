# Architecture de nwidgets

## Vue d'ensemble

nwidgets est un système de widgets Wayland construit avec GPUI, organisé en modules clairs et maintenables.

## Structure du projet

```
src/
├── main.rs              # Point d'entrée minimal
├── nwidgets.rs          # Orchestration principale
├── types.rs             # Types et constantes partagés
│
├── app/                 # Logique applicative
│   ├── init.rs         # Initialisation des services
│   ├── keys.rs         # Keybindings globaux
│   └── subscriptions.rs # Gestion des événements
│
├── windows/             # Gestion des fenêtres
│   ├── panel.rs        # Barre supérieure
│   ├── chat.rs         # Fenêtre de chat
│   └── launcher.rs     # Lanceur d'applications
│
├── services/            # Services système
│   ├── system/         # Services système (Hyprland, D-Bus)
│   ├── media/          # Services média (Audio, MPRIS)
│   ├── hardware/       # Services hardware (Bluetooth, Monitoring)
│   ├── ui/             # Services UI (Chat, Notifications, OSD)
│   ├── network/        # Services réseau
│   ├── cef/            # Chromium Embedded Framework
│   └── launcher/       # Logique du lanceur
│
├── widgets/             # Composants UI complexes
│   ├── panel/          # Modules du panel
│   ├── control_center/ # Centre de contrôle
│   ├── chat.rs         # Widget chat
│   ├── launcher.rs     # Widget lanceur
│   ├── notifications.rs # Widget notifications
│   └── osd.rs          # On-Screen Display
│
├── ui/                  # Composants UI réutilisables
│   └── components/     # Composants de base
│
├── assets.rs            # Gestion des assets et icônes
├── cli.rs               # Interface CLI
├── logger.rs            # Configuration des logs
└── theme.rs             # Thème visuel
```

## Patterns architecturaux

### 1. Services globaux (GPUI Entity)

Les services sont des singletons GPUI accessibles via `Service::global(cx)` :

```rust
let service = AudioService::global(cx);
service.read(cx).volume;
service.update(cx, |s, cx| s.set_volume(50, cx));
```

### 2. Event-driven architecture

Les services émettent des événements via `EventEmitter` :

```rust
impl EventEmitter<VolumeChanged> for AudioService {}

cx.subscribe(&service, |service, event: &VolumeChanged, cx| {
    // Réagir au changement
}).detach();
```

### 3. Window management

Chaque fenêtre gère son propre état via `OnceCell` :

```rust
static WINDOW: OnceCell<Arc<Mutex<WindowHandle<Widget>>>> = OnceCell::new();

pub fn open(cx: &mut App) {
    let window = cx.open_window(...);
    WINDOW.set(Arc::new(Mutex::new(window))).ok();
}
```

### 4. Separation of concerns

- **app/** : Logique d'initialisation et de coordination
- **windows/** : Gestion des fenêtres et leurs événements
- **services/** : Logique métier et intégration système
- **widgets/** : Composants UI complexes avec état
- **ui/components/** : Composants UI réutilisables sans état

## Flux de données

```
User Input → Window → Service → Event → Subscribers → UI Update
```

1. L'utilisateur interagit avec une fenêtre
2. La fenêtre appelle un service
3. Le service émet un événement
4. Les subscribers réagissent
5. L'UI se met à jour

## Ajout d'une nouvelle fonctionnalité

### Nouveau service

1. Créer `src/services/domain/my_service.rs`
2. Implémenter `EventEmitter` pour les événements
3. Ajouter `init()` et `global()` si nécessaire
4. Exporter dans `src/services/domain/mod.rs`
5. Initialiser dans `app/init.rs`

### Nouvelle fenêtre

1. Créer `src/windows/my_window.rs`
2. Implémenter `open()` et les event handlers
3. Ajouter dans `windows/mod.rs`
4. Appeler depuis `nwidgets.rs`
5. Setup subscriptions dans `app/subscriptions.rs`

### Nouveau widget

1. Créer `src/widgets/my_widget.rs`
2. Implémenter `Render` trait
3. Utiliser les services via `Service::global(cx)`
4. Exporter dans `widgets/mod.rs`

## Optimisations

- **Event-driven** : Pas de polling, uniquement des réactions aux événements
- **Lazy loading** : Les services ne s'activent que quand nécessaire
- **String caching** : `SharedString` pour éviter les allocations
- **Deferred rendering** : Rendu asynchrone pour les vues complexes

## Tests

```bash
cargo test                    # Tests unitaires
cargo test --test integration # Tests d'intégration
```

## Debugging

```bash
RUST_LOG=debug cargo run      # Logs détaillés
RUST_LOG=nwidgets=trace cargo run # Logs très détaillés
```
