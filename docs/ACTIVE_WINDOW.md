# Module Active Window

## Fonctionnement

Le module `ActiveWindowModule` affiche les informations de la fenêtre active Hyprland :
- **Icône** de l'application (32px)
- **Classe** de la fenêtre (nom de l'app)
- **Titre** de la fenêtre (tronqué à 30 caractères)

## Convention de nommage des icônes

**Principe simple** : Le nom d'icône correspond à la **classe de la fenêtre en minuscules**.

### Exemples

```rust
// Classe Hyprland → Nom d'icône
"firefox"         → assets/firefox.svg
"discord"         → assets/discord.svg
"spotify"         → assets/spotify.svg
"steam"           → assets/steam.svg
"kitty"           → assets/kitty.svg
```

### Cas spéciaux

Certaines classes ont des mappings spéciaux pour plus de cohérence :

```rust
// Variantes de navigateurs
"zen-twilight" | "zen-alpha" | "zen" → "firefox"
"vesktop"                             → "discord"

// Noms de domaine inversés (org.app.Name)
"net.lutris.lutris"        → "lutris"
"org.keepassxc.keepassxc"  → "keepassxc"
"org.gnome.nautilus"       → "file-manager"
"org.inkscape.inkscape"    → "inkscape"
"dev.zed.zed"              → "zeditor"

// Terminaux → icône générique
"kitty" | "alacritty" | "wezterm" | "foot" → "terminal"
```

## Ajout d'une nouvelle icône d'app

### Méthode 1 : Icône directe (recommandé)

1. **Nommer le SVG selon la classe Hyprland**
   ```bash
   # Exemple pour une app avec la classe "obsidian"
   cp mon-icone.svg assets/obsidian.svg
   ```

2. **C'est tout !** L'icône sera chargée automatiquement

### Méthode 2 : Mapping personnalisé

Si le nom de classe ne correspond pas au nom d'icône souhaité :

1. **Ajouter le SVG** dans `assets/`
   ```bash
   cp mon-icone.svg assets/mon-app.svg
   ```

2. **Ajouter le mapping** dans `src/widgets/panel/modules/active_window.rs`
   ```rust
   fn get_icon_name(class: &str) -> String {
       let normalized = class.to_lowercase();
       match normalized.as_str() {
           "ma-classe-hyprland" => "mon-app".to_string(),
           // ... autres mappings
       }
   }
   ```

## Détection de nouvelles apps

Pour identifier la classe d'une fenêtre Hyprland :

```bash
# Lister toutes les fenêtres avec leurs classes
hyprctl clients | grep class

# Ou en JSON
hyprctl clients -j | jq '.[].class'

# Active window uniquement
hyprctl activewindow -j | jq '.class'
```

## Fallback

Si aucune icône n'est trouvée pour une classe :
- **Fallback** : `assets/nixos.svg` (placeholder)
- **Affichage** : "NixOS - Nia"

## Structure du module

```rust
ActiveWindowModule {
    // Abonnement au service Hyprland
    hyprland: Entity<HyprlandService>,
}
```

Le module se met à jour automatiquement via l'événement `ActiveWindowChanged`.

## Layout

Position : **Gauche du panel**

```
┌─────────────────────────────────────────────┐
│ [Icon] App                                  │
│        Window Title                         │
└─────────────────────────────────────────────┘
```

- **Largeur** : 350px min, 450px max
- **Hauteur d'icône** : 32px
- **Classe** : Semi-bold, $snow1
- **Titre** : Small, $polar4/$snow2 blend
