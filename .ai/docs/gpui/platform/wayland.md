# Wayland Support

GPUI's native Wayland integration for Linux desktop environments.

## Features

- **Native Wayland Protocol**: Direct protocol implementation
- **Layer Shell Support**: Desktop widgets and overlays
- **High DPI Support**: Automatic scaling
- **Clipboard Integration**: Wayland clipboard protocol
- **Input Methods**: XKB keyboard handling

## Setup

### NixOS Dependencies
```nix
# shell.nix
{ pkgs ? import <nixpkgs> {} }:
pkgs.mkShell {
  buildInputs = with pkgs; [
    wayland
    libxkbcommon
    vulkan-loader
  ];
}
```

### Cargo Features
```toml
[dependencies]
gpui = { version = "0.2.2", features = ["wayland"] }
```

## Layer Shell Windows

Create desktop widgets and overlays:

```rust
#[cfg(all(target_os = "linux", feature = "wayland"))]
use gpui::platform::linux::wayland::{LayerShell, Layer, Anchor};

#[cfg(all(target_os = "linux", feature = "wayland"))]
fn create_desktop_widget(cx: &mut App) {
    let options = LayerShellOptions {
        layer: Layer::Overlay,
        anchor: Anchor::TOP | Anchor::RIGHT,
        exclusive_zone: 0,
        margin: Margin::uniform(10),
        keyboard_interactivity: KeyboardInteractivity::None,
    };
    
    cx.open_layer_shell_window(options, |cx| {
        DesktopWidget::new()
    });
}
```

## Window Management

### Window Types
- **Normal Windows**: Standard application windows
- **Layer Shell**: Desktop widgets, panels, notifications
- **Popup Windows**: Context menus, tooltips

### Window Properties
```rust
let options = WindowOptions {
    wayland_app_id: Some("com.example.myapp".into()),
    title: Some("My App".into()),
    ..Default::default()
};
```

## Input Handling

### Keyboard
- XKB keymap support
- Compose key sequences
- Multiple keyboard layouts

### Mouse and Touch
- Multi-touch gestures
- High-precision scrolling
- Tablet input support

## Graphics

### Rendering Backend
- Vulkan primary backend
- OpenGL ES fallback
- Hardware acceleration

### Buffer Management
- Efficient buffer sharing
- Damage tracking
- VSync support

## Environment Variables

```bash
# Force Wayland backend
export WAYLAND_DISPLAY=wayland-0

# Enable debug logging
export WAYLAND_DEBUG=1

# Set scaling factor
export GDK_SCALE=2
```

## Compositor Compatibility

### Tested Compositors
- **GNOME Shell** (Mutter)
- **KDE Plasma** (KWin)
- **Sway**
- **Hyprland**
- **River**

### Protocol Support
- `wl_compositor`
- `wl_surface`
- `wl_shell_surface`
- `xdg_shell`
- `wlr_layer_shell_unstable_v1`
- `wp_fractional_scale_v1`

## Best Practices

1. **Test Multiple Compositors**: Different behavior across compositors
2. **Handle Protocol Errors**: Graceful fallbacks
3. **Respect Compositor Hints**: DPI, scaling, decorations
4. **Use Layer Shell Appropriately**: Only for desktop integration
5. **Monitor Protocol Extensions**: New features and capabilities
